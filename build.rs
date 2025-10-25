fn main() -> Result<(), wdk_build::ConfigError> {
   println!("cargo:rustc-link-lib=NtosKernel");
   wdk_build::configure_wdk_binary_build()
}