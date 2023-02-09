use lazy_static::lazy_static;

lazy_static! {
    pub static ref JWT_SECRET: String = dotenv_codegen::dotenv!("JWT_SECRET").to_owned();
}
