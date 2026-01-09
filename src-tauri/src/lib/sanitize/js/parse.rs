use swc_common::{sync::Lrc, FileName, Globals, SourceMap, GLOBALS};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

pub fn parse_js(text: &str) -> Result<(), ()> {
    let cm = Lrc::new(SourceMap::default());
    let globals = Globals::new();
    GLOBALS.set(&globals, || {
        if parse_module(cm.clone(), text).is_ok() {
            return Ok(());
        }
        parse_script(cm.clone(), text)
    })
}

fn parse_module(cm: Lrc<SourceMap>, text: &str) -> Result<(), ()> {
    let fm = cm.new_source_file(
        FileName::Custom("clipboard.js".to_string()).into(),
        text.to_string(),
    );
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    parser.parse_module().map_err(|_| ())?;
    if !parser.take_errors().is_empty() {
        return Err(());
    }
    Ok(())
}

fn parse_script(cm: Lrc<SourceMap>, text: &str) -> Result<(), ()> {
    let fm = cm.new_source_file(
        FileName::Custom("clipboard.js".to_string()).into(),
        text.to_string(),
    );
    let lexer = Lexer::new(
        Syntax::Es(Default::default()),
        Default::default(),
        StringInput::from(&*fm),
        None,
    );
    let mut parser = Parser::new_from(lexer);
    parser.parse_script().map_err(|_| ())?;
    if !parser.take_errors().is_empty() {
        return Err(());
    }
    Ok(())
}
