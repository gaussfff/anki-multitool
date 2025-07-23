use anyhow::Result;
use std::{fmt, fs::File, marker::PhantomData};

pub struct FileCommitBuffer<PA, C, D, PD>
where
    PA: AsyncFn(D) -> Result<PD>,
    C: AsyncFn(String) -> Result<()>,
    PD: fmt::Display,
{
    tmpfile: File,
    prepare_action: PA,
    commit: C,
    _type: PhantomData<D>,
}

impl<D, PA: AsyncFn(D) -> Result<PD>, C: AsyncFn(String) -> Result<()>, PD: fmt::Display>
    FileCommitBuffer<PA, C, D, PD>
{
    pub fn new(prepare_action: PA, commit: C) -> Result<Self> {
        use tempfile::tempfile;

        Ok(Self {
            tmpfile: tempfile()?,
            prepare_action,
            commit,
            _type: PhantomData,
        })
    }

    pub async fn exec_and_commit(&mut self, consuming_data: impl Iterator<Item = D>) -> Result<()> {
        use std::io::Write;

        for data in consuming_data {
            writeln!(self.tmpfile, "{}", (self.prepare_action)(data).await?)?;
        }

        self.commit().await
    }

    async fn commit(&mut self) -> Result<()> {
        use std::io::Seek;
        use std::io::{BufRead, BufReader};

        self.tmpfile.rewind()?;

        for line in BufReader::new(&self.tmpfile).lines() {
            (self.commit)(line?).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::Write,
        sync::{Arc, Mutex},
    };
    use tempfile::NamedTempFile;

    #[tokio::test]
    pub async fn test_file_commit_buffer() {
        use std::io::{BufRead, BufReader, Seek};

        let file = Arc::new(Mutex::new(
            NamedTempFile::new().expect("failed to create temp file"),
        ));

        FileCommitBuffer::new(
            async |data| Ok(format!("prepared {data}")),
            async |data| {
                writeln!(
                    Arc::clone(&file).lock().expect("failed to get file"),
                    "{data}"
                )?;
                Ok(())
            },
        )
        .expect("failed to create FileCommitBuffer")
        .exec_and_commit(vec!["line1", "line2", "line3", "line4", "line5"].into_iter())
        .await
        .expect("failed to commit data");

        file.lock()
            .expect("failed to get file")
            .rewind()
            .expect("failed to rewind temp file");

        let file_guard = file.lock().expect("failed to get file");
        let mut lines = BufReader::new(file_guard.as_file()).lines();

        assert_eq!(lines.next().unwrap().unwrap(), "prepared line1");
        assert_eq!(lines.next().unwrap().unwrap(), "prepared line2");
        assert_eq!(lines.next().unwrap().unwrap(), "prepared line3");
        assert_eq!(lines.next().unwrap().unwrap(), "prepared line4");
        assert_eq!(lines.next().unwrap().unwrap(), "prepared line5");
    }

    #[tokio::test]
    pub async fn test_failed_file_commit_buffer() {
        use std::fs::metadata;
        use std::sync::atomic::{AtomicU8, Ordering};

        let file = Arc::new(Mutex::new(
            NamedTempFile::new().expect("failed to create temp file"),
        ));
        let counter = Arc::new(AtomicU8::new(0));

        assert!(
            FileCommitBuffer::new(
                async |data| {
                    let counter = Arc::clone(&counter);

                    if counter.load(Ordering::Relaxed) > 2 {
                        return Err(anyhow::anyhow!("simulated failure on commit"));
                    }

                    counter.fetch_add(1, Ordering::Relaxed);
                    Ok(data)
                },
                async |data| {
                    writeln!(
                        Arc::clone(&file).lock().expect("failed to get file"),
                        "{data}"
                    )?;
                    Ok(())
                }
            )
            .expect("failed to create FileCommitBuffer")
            .exec_and_commit(vec!["line1", "line2", "line3", "line4", "line5"].into_iter())
            .await
            .is_err()
        );

        assert_eq!(
            metadata(file.lock().expect("failed to get file").path())
                .expect("failed to get metadata of file")
                .len(),
            0
        );
    }
}
