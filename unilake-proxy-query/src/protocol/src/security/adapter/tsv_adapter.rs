use async_trait::async_trait;
use casbin::{Adapter, Filter, Model, Result};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};

pub struct TsvFileAdapter<P> {
    file_path: P,
    is_filtered: bool,
}

impl<P> TsvFileAdapter<P>
where
    P: AsRef<Path> + Send + Sync,
{
    pub fn new(p: P) -> TsvFileAdapter<P> {
        TsvFileAdapter {
            file_path: p,
            is_filtered: false,
        }
    }

    async fn load_policy_file(&mut self, m: &mut dyn Model) -> Result<()> {
        let f = tokio::fs::File::open(&self.file_path).await?;
        let mut lines = BufReader::new(f).lines();
        while let Some(line) = lines.next_line().await? {
            self.handler(line, m)
        }
        Ok(())
    }

    fn handler(&self, line: String, m: &mut dyn Model) {
        let parts: Vec<String> = line
            .split_terminator('\t')
            .map(|x| x.to_string())
            .filter(|x| !x.starts_with("#"))
            .collect();
        let key = &parts[0];
        if let Some(ref sec) = key.chars().next().map(|x| x.to_string()) {
            if let Some(ast_map) = m.get_mut_model().get_mut(sec) {
                if let Some(ast) = ast_map.get_mut(key) {
                    ast.policy.insert(parts[1..].to_vec());
                }
            }
        }
    }

    #[allow(unused)]
    async fn load_filtered_policy_file<'a>(
        &self,
        _m: &mut dyn Model,
        _filter: Filter<'a>,
    ) -> Result<bool> {
        todo!()
    }

    #[allow(unused)]
    async fn save_policy_file(&self, _text: String) -> Result<()> {
        todo!()
    }
}

#[async_trait]
impl<P> Adapter for TsvFileAdapter<P>
where
    P: AsRef<Path> + Send + Sync,
{
    async fn load_policy(&mut self, m: &mut dyn Model) -> Result<()> {
        self.is_filtered = false;
        self.load_policy_file(m).await?;
        Ok(())
    }

    async fn load_filtered_policy<'a>(&mut self, _m: &mut dyn Model, _f: Filter<'a>) -> Result<()> {
        todo!()
    }

    async fn save_policy(&mut self, _m: &mut dyn Model) -> Result<()> {
        todo!()
    }

    async fn clear_policy(&mut self) -> Result<()> {
        todo!()
    }

    fn is_filtered(&self) -> bool {
        self.is_filtered
    }

    async fn add_policy(&mut self, _sec: &str, _ptype: &str, _rule: Vec<String>) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }

    async fn add_policies(
        &mut self,
        _sec: &str,
        _ptype: &str,
        _rules: Vec<Vec<String>>,
    ) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }

    async fn remove_policy(
        &mut self,
        _sec: &str,
        _ptype: &str,
        _rule: Vec<String>,
    ) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }

    async fn remove_policies(
        &mut self,
        _sec: &str,
        _ptype: &str,
        _rule: Vec<Vec<String>>,
    ) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }

    async fn remove_filtered_policy(
        &mut self,
        _sec: &str,
        _ptype: &str,
        _field_index: usize,
        _field_values: Vec<String>,
    ) -> Result<bool> {
        // this api shouldn't implement, just for convenience
        Ok(true)
    }
}
