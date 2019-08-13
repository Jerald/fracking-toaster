use serenity::framework::standard::{
    Args,
};

use cylon_ast::CylonRoot;

#[derive(Debug)]
pub struct YololConfig
{
    pub input: InputFlag,
    pub output: OutputFlag,
}

#[derive(Debug)]
pub enum InputFlag
{
    Yolol,
    CylonAst
}

// #[derive(Debug)]
pub enum YololInput
{
    Yolol(String),
    CylonAst(CylonRoot)
}

#[derive(Debug)]
pub enum OutputFlag
{
    Execution,
    Yolol,
    CylonAst,
    Ast,
    Tokens
}

impl YololConfig
{
    pub fn new() -> Self
    {
        YololConfig {
            input: InputFlag::Yolol,
            output: OutputFlag::Execution,
        }
    }

    pub fn parse_args(args: &mut Args) -> Result<Self, String>
    {
        let mut config = YololConfig::new();

        loop
        {
            let current = match args.current()
            {
                Some(args) => args,
                None => return Err("Unable to get an argument! Might not have supplied any args...".to_owned())
            };

            println!("Current arg: {}", current);

            match current
            {
                "--input=cylon_ast" |
                "-ic" => config.input = InputFlag::CylonAst,

                "--output=yolol" |
                "--output=code" |
                "-oy" => config.output = OutputFlag::Yolol,

                "--output=cylon_ast" |
                "-oc" => config.output = OutputFlag::CylonAst,

                "--output=ast" |
                "--output=parsed" |
                "-oa" => config.output = OutputFlag::Ast,

                "--output=tokens" |
                "-ot" => config.output = OutputFlag::Tokens,

                _ => break
            }

            args.advance();
        }

        Ok(config)
    }
}

