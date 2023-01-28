use std::fs::File;
use std::io::Write;
use anyhow::Result;
use gumdrop::Options;
use std::process::Command;
use sailfish::TemplateOnce;
use tectonic::latex_to_pdf;


#[derive(Debug, Options)]
struct CropOptions {
    #[options(free)]
    input: String,

    #[options(no_short, help = "using `%%HiResBoundingBox` instead of `%%BoundingBox'")]
    hires: bool,

    #[options(help = "print help message")]
    help: bool,
}

#[derive(Debug, TemplateOnce)]
#[template(path = "crop.stpl")]
struct CropTemplate<'a> {
    input: String,
    pdfmajorversion: usize,
    pdfminorversion: usize,
    pages: Vec<&'a str>,
}

// todo: auto find ghostscript
fn find_ghostscript() -> Result<String> {
    Ok(String::from("gs"))
}

fn main() -> Result<()> {
    let opts = CropOptions::parse_args_default_or_exit();

    let gs_path = find_ghostscript()?;

    let gs_output = Command::new(gs_path).arg("-q")
        .arg("-dBATCH")
        .arg("-dNOPAUSE")
        .arg("-sDEVICE=bbox")
        .arg(&opts.input)
        .output()?;

    let gs_stderr = String::from_utf8_lossy(&gs_output.stderr);
    let pages: Vec<_> = {
        let prefix = if opts.hires { "%%HiResBoundingBox:" } else { "%%BoundingBox:" };
        gs_stderr.lines().filter_map(
            |line| if line.starts_with(prefix) {
                Some(line.trim_start_matches(prefix).trim())
            } else { None }
        ).collect()
    };

    // todo: get pdf version from pdf
    let template = CropTemplate {
        input: hex::encode_upper(&opts.input),
        pdfmajorversion: 1,
        pdfminorversion: 5,
        pages,
    };

    let rendered = template.render_once()?;

    // todo: process more carefully.
    if let Some((base, ext)) = opts.input.rsplit_once('.') {
        let output_pdf = latex_to_pdf(&rendered).expect("Compile Failed!");
        let output_path = format!("{}-crop.{}", base, ext);
        let mut output = File::create(&output_path)?;
        output.write_all(&output_pdf)?;
        output.flush()?;
    }

    Ok(())
}
