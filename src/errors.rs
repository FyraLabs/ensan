#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HCL error: {0}")]
    Hcl(#[from] hcl::Error),
    #[error("Hcl Eval errors: {0}")]
    HclEvals(#[from] hcl::eval::Errors),
    #[error("Hcl Eval error: {0}")]
    HclEval(#[from] hcl::eval::Error),
}
