use lazy_static::lazy_static;

lazy_static! {
    pub static ref SECRET_KEY: String = dotenv_codegen::dotenv!("SECRET_KEY").to_owned();
    pub static ref REDIS_URL: String = dotenv_codegen::dotenv!("REDIS_URL").to_owned();
    pub static ref SMS_HOST: String = dotenv_codegen::dotenv!("SMS_HOST").to_owned();
    pub static ref DATABASE_URL: String = dotenv_codegen::dotenv!("DATABASE_URL").to_owned();
    pub static ref APP_NAME: String = dotenv_codegen::dotenv!("APP_NAME").to_owned();
}
