use std::env;

use anyhow::Result;

/// -stage <stage>: Specify the stage of an entry-point function.
pub enum Stage {
    Vertex,
    Hull,
    Domain,
    Geometry,
    Fragment,
    Compute,
    RayGeneration,
    Intersection,
    AnyHit,
    ClosestHit,
    Miss,
    Callable,
    Mesh,
    Amplification,
}

impl Stage {
    pub fn to_str(&self) -> &str {
        match self {
            Stage::Vertex => "vertex",
            Stage::Hull => "hull",
            Stage::Domain => "domain",
            Stage::Geometry => "geometry",
            Stage::Fragment => "fragment",
            Stage::Compute => "compute",
            Stage::RayGeneration => "raygeneration",
            Stage::Intersection => "intersection",
            Stage::AnyHit => "anyhit",
            Stage::ClosestHit => "closesthit",
            Stage::Miss => "miss",
            Stage::Callable => "callable",
            Stage::Mesh => "mesh",
            Stage::Amplification => "amplification",
        }
    }
}

pub struct CompileShaderOptions<'a> {
    /// -stage <stage>: Specify the stage of an entry-point function.
    pub stage: Option<Stage>,
    /// -profile <profile>: Specify the target profile.
    pub profile: Option<&'a str>,
    /// -entry <entry-point>: Specify the entry-point function.
    pub entry_point: Option<&'a str>,
    /// -target <target>: Specify the target language.
    pub target: Option<&'a str>,
    /// File to compile
    pub file: &'a str,
}

const SLANGC_BIN_PATH: &'static str = env!("SLANGC_BIN_PATH");

pub fn compile_shader(options: &CompileShaderOptions) -> Result<Vec<u8>> {
    let mut command = std::process::Command::new(SLANGC_BIN_PATH);

    if let Some(stage) = options.stage.as_ref() {
        command.arg("-stage").arg(stage.to_str());
    }
    if let Some(profile) = options.profile {
        command.arg("-profile").arg(profile);
    }
    if let Some(entry_point) = options.entry_point {
        command.arg("-entry").arg(entry_point);
    }
    if let Some(target) = options.target {
        command.arg("-target").arg(target);
    }

    command.arg(options.file);

    let output = command.output().unwrap();
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            String::from_utf8(output.stderr.clone()).unwrap()
        ));
    }

    Ok(output.stdout)
}

pub fn print_help_info() -> Result<()> {
    let output = std::process::Command::new(SLANGC_BIN_PATH)
        .arg("-help")
        .output()?;
    println!("slangc help: {}", String::from_utf8_lossy(&output.stderr));
    Ok(())
}

#[test]
fn test_compile_shader() {
    use rspirv::binary::Disassemble;

    let data = match compile_shader(&CompileShaderOptions {
        stage: Some(Stage::Vertex),
        profile: Some("spirv_1_6"),
        entry_point: Some("vertexMain"),
        target: Some("spirv"),
        file: "test-shaders/bindless_triangle.slang",
    }) {
        Ok(data) => data,
        Err(e) => {
            panic!("{}", e);
        }
    };

    let mut loader = rspirv::dr::Loader::new();
    rspirv::binary::parse_bytes(&data, &mut loader).unwrap();
    let module = loader.module();
    let dis = module.disassemble();

    assert!(dis.contains("OpCapability Shader"));
    assert!(dis.contains("vertexMain"));
    assert!(dis.contains("Version: 1.6"));
}
