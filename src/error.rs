use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    FromHexError(#[from] uint::FromHexError),
    #[error(transparent)]
    FromHexError2(#[from] rustc_hex::FromHexError),
    #[error(transparent)]
    WalletError(#[from] ethers::prelude::WalletError),
    #[error(transparent)]
    ProviderError(#[from] ethers::prelude::ProviderError),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
}
