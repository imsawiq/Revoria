use std::collections::HashSet;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, exit};
use std::{env, fs};

fn main() {
    println!("cargo::rerun-if-changed=.env");
    println!("cargo::rerun-if-changed=.env.local");
    println!("cargo::rerun-if-changed=.env.staging");
    println!("cargo::rerun-if-changed=.env.prod");
    println!("cargo::rerun-if-changed=java/gradle");
    println!("cargo::rerun-if-changed=java/src");
    println!("cargo::rerun-if-changed=java/build.gradle.kts");
    println!("cargo::rerun-if-changed=java/settings.gradle.kts");
    println!("cargo::rerun-if-changed=java/gradle.properties");

    set_env();
    build_java_jars();
}

fn set_env() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    let mut exported = HashSet::<String>::new();

    let mut loaded_any = false;
    for file in [".env.local", ".env", ".env.staging", ".env.prod"] {
        let path = manifest_dir.join(file);
        if !path.exists() {
            continue;
        }

        loaded_any = true;
        for (var_name, var_value) in dotenvy::from_path_iter(&path)
            .into_iter()
            .flatten()
            .flatten()
        {
            if var_name == "DATABASE_URL" {
                // The sqlx database URL is a build-time detail that should not be exposed to the crate
                continue;
            }

            println!("cargo::rustc-env={var_name}={var_value}");
            exported.insert(var_name);
        }
    }

    // If there is no env file, still allow building as long as required vars are present
    // in the process environment.
    if !loaded_any {
        for key in [
            "MODRINTH_API_URL",
            "MODRINTH_API_URL_V3",
            "MODRINTH_LAUNCHER_META_URL",
            "MODRINTH_SOCKET_URL",
            "MODRINTH_URL",
        ] {
            if let Ok(val) = env::var(key) {
                println!("cargo::rustc-env={key}={val}");
                exported.insert(key.to_string());
            }
        }
    }

    for key in [
        "MODRINTH_API_URL",
        "MODRINTH_API_URL_V3",
        "MODRINTH_LAUNCHER_META_URL",
        "MODRINTH_SOCKET_URL",
        "MODRINTH_URL",
    ] {
        if env::var_os(key).is_none() && !exported.contains(key) {
            println!(
                "cargo::error=Missing required environment variable `{key}`. Create `packages/app-lib/.env.local` (or set the variable in your shell) and re-run the build."
            );
            exit(1);
        }
    }

    // Preserve any other variables from the .env file(s) for the crate.
    // (We already exported them above; this loop is only needed when using dotenvy::dotenv_iter().)
    if false {
        // unreachable; left intentionally to avoid changing behavior via comments
    }

    // NOTE: We intentionally do not call dotenvy::dotenv_iter() directly here because
    // the crate relies on compile-time env!() and we need deterministic file selection.

    return;
}

fn build_java_jars() {
    let out_dir =
        dunce::canonicalize(PathBuf::from(env::var_os("OUT_DIR").unwrap()))
            .unwrap();

    println!(
        "cargo::rustc-env=JAVA_JARS_DIR={}",
        out_dir.join("java/libs").display()
    );

    let gradle_path = fs::canonicalize(
        #[cfg(target_os = "windows")]
        "java\\gradlew.bat",
        #[cfg(not(target_os = "windows"))]
        "java/gradlew",
    )
    .unwrap();

    let mut build_dir_str = OsString::from("-Dorg.gradle.project.buildDir=");
    build_dir_str.push(out_dir.join("java"));

    let mut cmd = Command::new(gradle_path);
    #[cfg(target_os = "windows")]
    {
        if env::var_os("GRADLE_USER_HOME").is_none() {
            cmd.env("GRADLE_USER_HOME", r"C:\gradle-home");
        }
    }

    let exit_status = cmd
        .arg(build_dir_str)
        .arg("build")
        .arg("--no-daemon")
        .arg("--console=rich")
        .current_dir(dunce::canonicalize("java").unwrap())
        .status()
        .expect("Failed to wait on Gradle build");

    if !exit_status.success() {
        println!("cargo::error=Gradle build failed with {exit_status}");
        exit(exit_status.code().unwrap_or(1));
    }
}
