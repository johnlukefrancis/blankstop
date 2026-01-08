use swc_common::{sync::Lrc, FileName, Globals, SourceMap, GLOBALS};
use swc_ecma_ast::{Module, Script};
use swc_ecma_codegen::{text_writer::JsWriter, Config as CodegenConfig, Emitter};
use swc_ecma_parser::{EsConfig, Parser, StringInput, Syntax};

pub fn parse_and_codegen(text: &str) -> Result<String, ()> {
    let cm = Lrc::new(SourceMap::default());
    let globals = Globals::new();
    GLOBALS.set(&globals, || {
        if let Ok(module) = parse_module(cm.clone(), text) {
            return emit_module(cm, &module);
        }
        let script = parse_script(cm.clone(), text).map_err(|_| ())?;
        emit_script(cm, &script)
    })
}

fn parse_module(cm: Lrc<SourceMap>, text: &str) -> Result<Module, ()> {
    let fm = cm.new_source_file(FileName::Custom("clipboard.js".to_string()), text.to_string());
    let mut parser = Parser::new(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        StringInput::from(&*fm),
        None,
    );
    let module = parser.parse_module().map_err(|_| ())?;
    if !parser.take_errors().is_empty() {
        return Err(());
    }
    Ok(module)
}

fn parse_script(cm: Lrc<SourceMap>, text: &str) -> Result<Script, ()> {
    let fm = cm.new_source_file(FileName::Custom("clipboard.js".to_string()), text.to_string());
    let mut parser = Parser::new(
        Syntax::Es(EsConfig {
            jsx: true,
            ..Default::default()
        }),
        StringInput::from(&*fm),
        None,
    );
    let script = parser.parse_script().map_err(|_| ())?;
    if !parser.take_errors().is_empty() {
        return Err(());
    }
    Ok(script)
}

fn emit_module(cm: Lrc<SourceMap>, module: &Module) -> Result<String, ()> {
    let mut buf = Vec::new();
    {
        let mut emitter = Emitter {
            cfg: CodegenConfig {
                minify: false,
                ..Default::default()
            },
            comments: None,
            cm: cm.clone(),
            wr: JsWriter::new(cm, "\n", &mut buf, None),
        };
        emitter.emit_module(module).map_err(|_| ())?;
    }
    let output = String::from_utf8(buf).map_err(|_| ())?;
    Ok(trim_trailing_newlines(output))
}

fn emit_script(cm: Lrc<SourceMap>, script: &Script) -> Result<String, ()> {
    let mut buf = Vec::new();
    {
        let mut emitter = Emitter {
            cfg: CodegenConfig {
                minify: false,
                ..Default::default()
            },
            comments: None,
            cm: cm.clone(),
            wr: JsWriter::new(cm, "\n", &mut buf, None),
        };
        emitter.emit_script(script).map_err(|_| ())?;
    }
    let output = String::from_utf8(buf).map_err(|_| ())?;
    Ok(trim_trailing_newlines(output))
}

fn trim_trailing_newlines(mut text: String) -> String {
    while matches!(text.chars().last(), Some('\n' | '\r')) {
        text.pop();
    }
    text
}
