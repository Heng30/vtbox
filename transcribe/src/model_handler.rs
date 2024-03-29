use anyhow::Result;
use reqwest::{Client, Proxy};

pub struct ModelHandler {
    model_name: String, // list of downloaded models
    models_dir: String, // path to the models directory
}

impl ModelHandler {
    pub fn new(model_name: &str, models_dir: &str) -> Result<ModelHandler> {
        let model_handler = ModelHandler {
            model_name: model_name.to_string(),
            models_dir: models_dir.to_string(),
        };

        if !model_handler.is_model_existing() {
            model_handler.setup_directory()?;
        }

        Ok(model_handler)
    }

    pub fn setup_directory(&self) -> Result<()> {
        let path = std::path::Path::new(&self.models_dir);
        if !path.exists() {
            std::fs::create_dir_all(path)?;
        }
        Ok(())
    }

    pub async fn download_model(&self, proxy_info: Option<(&str, u16)>) -> Result<()> {
        if !self.is_model_existing() {
            self.setup_directory()?;
        }
        download_model(&self.models_dir, &self.model_name, proxy_info).await
    }

    pub fn is_model_existing(&self) -> bool {
        std::fs::metadata(format!("{}/{}", self.models_dir, self.model_name)).is_ok()
    }

    pub fn get_model_dir(&self) -> String {
        format!("{}/{}", &self.models_dir, &self.model_name)
    }
}

pub async fn download_model(
    models_dir: &str,
    model_name: &str,
    proxy_info: Option<(&str, u16)>,
) -> Result<()> {
    let base_url = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main";
    let url = format!("{}/{}", base_url, model_name);

    let client = if let Some((ip, port)) = proxy_info {
        let proxy = Proxy::all(format!("socks5://{}:{}", ip, port))?;
        Client::builder().proxy(proxy).build()?
    } else {
        Client::new()
    };

    let response = client.get(&url).send().await?;

    let mut file = std::fs::File::create(format!("{}/{}", models_dir, model_name))?;
    let mut content = std::io::Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_model_exists_existent_path() {
        let path = std::path::Path::new("test_models/ggml-tiny.bin");
        if !path.exists() {
            let _ = std::fs::create_dir_all(path);
        }

        let test_model = ModelHandler::new("ggml-tiny.bin", "test_models/").unwrap();
        let result = test_model.is_model_existing();
        assert_eq!(result, true);
    }

    #[tokio::test]
    async fn test_setup_directory_happy_case() {
        let path = std::path::Path::new("test_models/ggml-tiny.bin");
        if !path.exists() {
            let _ = std::fs::create_dir_all(path);
        }

        let test_model = ModelHandler::new("ggml-tiny.bin", "test_models/").unwrap();
        let result = test_model.setup_directory();
        assert_eq!(result.is_ok(), true);
        let _ = std::fs::remove_dir_all("test_models/");
    }

    #[tokio::test]
    async fn test_download_model_happy_case() {
        fn prep_test_dir() {
            let path = std::path::Path::new("test_dir/");
            if !path.exists() {
                let _ = std::fs::create_dir_all(path);
            }
        }

        prep_test_dir();

        let model_handler = ModelHandler::new("ggml-tiny.bin", "test_dir/").unwrap();

        let _result = model_handler.download_model(None).await;

        let is_file_existing = match std::fs::metadata("test_dir/ggml-tiny.bin") {
            Ok(_) => true,
            Err(_) => false,
        };

        assert_eq!(is_file_existing, true);

        let _ = std::fs::remove_dir_all("test_dir/");
    }
}
