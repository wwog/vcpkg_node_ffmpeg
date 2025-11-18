use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct AddonPreparer {
    ffmpeg_source_dir: PathBuf,
    addon_src_dir: PathBuf,
    vcpkg_root: PathBuf,
    cp_file_dir: PathBuf,
}

impl AddonPreparer {
    pub fn new() -> Self {
        let base_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
            PathBuf::from(&manifest_dir)
        } else {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };
        
        let ffmpeg_source_dir = base_dir.join("ffmpeg");
        let addon_src_dir = base_dir.join("addon_src");
        let vcpkg_root = base_dir.join("vcpkg");
        let cp_file_dir = base_dir.join("src").join("cp_file");
        
        Self {
            ffmpeg_source_dir,
            addon_src_dir,
            vcpkg_root,
            cp_file_dir,
        }
    }
    
    /// Prepare addon source code
    pub fn prepare_addon_source(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Preparing Node.js addon source code...");
        
        if !self.addon_src_dir.exists() {
            fs::create_dir_all(&self.addon_src_dir)?;
            println!("✓ Created addon_src directory");
        }
        
        self.create_config_h()?;
        self.modify_opt_common_c()?;
        self.modify_ffmpeg_dec_c()?;
        self.copy_cp_files()?;
        
        println!("✓ Node.js addon source code preparation completed");
        Ok(())
    }
    
    /// Copy files from src/cp_file to addon_src
    fn copy_cp_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.cp_file_dir.exists() {
            return Err(format!("Source directory does not exist: {}", self.cp_file_dir.display()).into());
        }
        
        println!("Copying files from {} to {}...", self.cp_file_dir.display(), self.addon_src_dir.display());
        
        // List of files to copy
        let files_to_copy = ["binding.c", "ffmpeg.c", "utils.c"];
        
        for file_name in &files_to_copy {
            let source_file = self.cp_file_dir.join(file_name);
            let target_file = self.addon_src_dir.join(file_name);
            
            if !source_file.exists() {
                println!("⚠ {} not found in cp_file directory, skipping", file_name);
                continue;
            }
            
            fs::copy(&source_file, &target_file)?;
            println!("✓ Copied {} to {}", file_name, target_file.display());
        }
        
        Ok(())
    }
    
    /// Create config.h file (required for ffmpeg compilation)
    /// This file uses conditional compilation to support all platforms
    fn create_config_h(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_h_path = self.ffmpeg_source_dir.join("config.h");
        
        // Check if cross-platform config.h already exists
        if config_h_path.exists() {
            let existing_content = fs::read_to_string(&config_h_path)?;
            // Check if it's a cross-platform version (contains conditional compilation)
            if existing_content.contains("#ifdef _WIN32") || existing_content.contains("#if defined(_WIN32)") {
                println!("✓ config.h already exists (cross-platform version), skipping creation");
                return Ok(());
            } else {
                println!("⚠ config.h exists but is old platform-specific version, regenerating as cross-platform...");
            }
        }
        
        // Generate cross-platform config.h using conditional compilation to support all platforms
        let config_h_content = r#"/* config.h - Cross-platform configuration for Node.js addon */
#ifndef CONFIG_H
#define CONFIG_H

/* Platform detection and system-specific defines */
#ifdef _WIN32
/* Windows specific defines */
#define HAVE_IO_H 1
#define HAVE_UNISTD_H 0
#define HAVE_SYS_RESOURCE_H 0
#define HAVE_GETPROCESSTIMES 1
#define HAVE_GETPROCESSMEMORYINFO 1
#define HAVE_SETCONSOLECTRLHANDLER 1
#define HAVE_SYS_SELECT_H 0
#define HAVE_TERMIOS_H 0
#define HAVE_KBHIT 1
#define HAVE_PEEKNAMEDPIPE 1
#define HAVE_GETSTDHANDLE 1
#define HAVE_GETRUSAGE 0

/* Threading */
#define HAVE_PTHREADS 0
#define HAVE_W32THREADS 1

/* Math functions - MSVC provides these as intrinsics */
#define HAVE_LRINT 1
#define HAVE_LRINTF 1

/* System math library functions - MSVC provides some */
#define HAVE_CBRT 1
#define HAVE_CBRTF 1
#define HAVE_COPYSIGN 1
#define HAVE_ERF 1
#define HAVE_HYPOT 1
#define HAVE_RINT 1
#define HAVE_ROUND 1
#define HAVE_ROUNDF 1
#define HAVE_TRUNC 1
#define HAVE_TRUNCF 1
#define HAVE_ATANF 1
#define HAVE_ATAN2F 1
#define HAVE_POWF 1

/* Compiler identification */
#ifdef _MSC_VER
#define CC_IDENT "MSVC"
#else
#define CC_IDENT "GCC/Clang"
#endif

#else
/* Unix (macOS/Linux) specific defines */
#define HAVE_IO_H 0
#define HAVE_UNISTD_H 1
#define HAVE_SYS_RESOURCE_H 1
#define HAVE_GETPROCESSTIMES 0
#define HAVE_GETPROCESSMEMORYINFO 0
#define HAVE_SETCONSOLECTRLHANDLER 0
#define HAVE_SYS_SELECT_H 1
#define HAVE_TERMIOS_H 1
#define HAVE_KBHIT 0
#define HAVE_PEEKNAMEDPIPE 0
#define HAVE_GETSTDHANDLE 0
#define HAVE_GETRUSAGE 1

/* Threading */
#define HAVE_PTHREADS 1
#define HAVE_W32THREADS 0

/* Math functions */
#define HAVE_LRINT 1
#define HAVE_LRINTF 1

/* System math library functions (macOS/Linux have these) */
#define HAVE_CBRT 1
#define HAVE_CBRTF 1
#define HAVE_COPYSIGN 1
#define HAVE_ERF 1
#define HAVE_HYPOT 1
#define HAVE_RINT 1
#define HAVE_ROUND 1
#define HAVE_ROUNDF 1
#define HAVE_TRUNC 1
#define HAVE_TRUNCF 1
#define HAVE_ATANF 1
#define HAVE_ATAN2F 1
#define HAVE_POWF 1

/* Compiler identification */
#ifdef __clang__
#define CC_IDENT "Clang"
#elif defined(__GNUC__)
#define CC_IDENT "GCC"
#else
#define CC_IDENT "GCC/Clang"
#endif

#endif /* _WIN32 */

/* FFmpeg components */
#define CONFIG_AVUTIL 1
#define CONFIG_AVCODEC 1
#define CONFIG_AVFORMAT 1
#define CONFIG_AVDEVICE 1
#define CONFIG_AVFILTER 1
#define CONFIG_SWSCALE 1
#define CONFIG_SWRESAMPLE 1
#define CONFIG_POSTPROC 0

/* Architecture detection */
#define ARCH_X86_32 0

#ifdef _WIN32
/* Windows architecture */
#ifdef _M_ARM64
#define ARCH_X86_64 0
#define ARCH_AARCH64 1
#elif defined(_M_X64) || defined(_M_AMD64)
#define ARCH_X86_64 1
#define ARCH_AARCH64 0
#else
#define ARCH_X86_64 0
#define ARCH_AARCH64 0
#endif
#else
/* Unix architecture */
#ifdef __aarch64__
#define ARCH_X86_64 0
#define ARCH_AARCH64 1
#elif defined(__x86_64__) || defined(__amd64__)
#define ARCH_X86_64 1
#define ARCH_AARCH64 0
#else
#define ARCH_X86_64 0
#define ARCH_AARCH64 0
#endif
#endif

/* Endianness */
#define HAVE_BIGENDIAN 0

/* FFmpeg data directory - empty for Node.js addon */
#define FFMPEG_DATADIR ""
#define AVCONV_DATADIR ""

/* Build configuration */
#define CONFIG_THIS_YEAR 2025
#ifdef _WIN32
#define FFMPEG_CONFIGURATION "Cross-platform build for Node.js addon (Windows)"
#else
#define FFMPEG_CONFIGURATION "Cross-platform build for Node.js addon (Unix)"
#endif
#define FFMPEG_VERSION "N/A"

#endif /* CONFIG_H */
"#;
        
        fs::write(&config_h_path, config_h_content)?;
        println!("✓ config.h created (cross-platform version): {}", config_h_path.display());
        Ok(())
    }
    
    /// Modify opt_common.c to add conditional compilation for postproc
    fn modify_opt_common_c(&self) -> Result<(), Box<dyn std::error::Error>> {
        let opt_common_c_path = self.ffmpeg_source_dir.join("fftools").join("opt_common.c");
        
        if !opt_common_c_path.exists() {
            println!("⚠ opt_common.c not found, skipping modification");
            return Ok(());
        }
        
        let content = fs::read_to_string(&opt_common_c_path)?;
        
        // Check if already modified
        if content.contains("#if CONFIG_POSTPROC") && content.contains("PRINT_LIB_INFO(postproc") {
            println!("✓ opt_common.c already modified, skipping");
            return Ok(());
        }
        
        // Find postproc line in print_all_libs_info function
        let pattern = "    PRINT_LIB_INFO(postproc,   POSTPROC,   flags, level);";
        if let Some(pos) = content.find(pattern) {
            let before = &content[..pos];
            let after = &content[pos + pattern.len()..];
            
            let modified = format!("{}#if CONFIG_POSTPROC\n    PRINT_LIB_INFO(postproc,   POSTPROC,   flags, level);\n#endif{}", 
                before, after);
            
            fs::write(&opt_common_c_path, modified)?;
            println!("✓ opt_common.c modified: added CONFIG_POSTPROC conditional compilation");
        } else {
            println!("⚠ Could not find postproc line in opt_common.c, skipping modification");
        }
        
        Ok(())
    }
    
    /// Modify ffmpeg_dec.c to use ffmpeg's compat stdbit.h instead of system stdbit.h
    /// and add MSVC compatibility for _Generic macro
    fn modify_ffmpeg_dec_c(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ffmpeg_dec_c_path = self.ffmpeg_source_dir.join("fftools").join("ffmpeg_dec.c");
        
        if !ffmpeg_dec_c_path.exists() {
            println!("⚠ ffmpeg_dec.c not found, skipping modification");
            return Ok(());
        }
        
        let content = fs::read_to_string(&ffmpeg_dec_c_path)?;
        
        // Check if already modified
        if content.contains("#include \"compat/stdbit/stdbit.h\"") && 
           content.contains("/* MSVC compatibility for stdbit functions */") {
            println!("✓ ffmpeg_dec.c already modified, skipping");
            return Ok(());
        }
        
        let mut modified = content.clone();
        
        // Replace system stdbit.h with ffmpeg's compat version
        if modified.contains("#include <stdbit.h>") {
            modified = modified.replace(
                "#include <stdbit.h>",
                "#include \"compat/stdbit/stdbit.h\""
            );
        }
        
        // Add MSVC compatibility macro definitions (MSVC doesn't support _Generic)
        // Add MSVC-specific compatibility definitions after including stdbit.h
        if modified.contains("#include \"compat/stdbit/stdbit.h\"") {
            let msvc_compat = r#"
/* MSVC compatibility for stdbit functions - MSVC doesn't support _Generic */
#ifdef _MSC_VER
/* Undefine the _Generic-based macros from compat header */
#undef stdc_count_ones
#undef stdc_trailing_zeros
/* Provide explicit implementations for unsigned int (used in this file) */
static inline unsigned int stdc_count_ones_ui_compat(unsigned int value) {
    unsigned int count = 0;
    while (value) {
        count += value & 1;
        value >>= 1;
    }
    return count;
}
#define stdc_count_ones(value) stdc_count_ones_ui_compat((unsigned int)(value))

static inline unsigned int stdc_trailing_zeros_ui_compat(unsigned int value) {
    if (!value) return sizeof(unsigned int) * 8;
    unsigned int count = 0;
    while ((value & 1) == 0) {
        value >>= 1;
        count++;
    }
    return count;
}
#define stdc_trailing_zeros(value) stdc_trailing_zeros_ui_compat((unsigned int)(value))
#endif /* _MSC_VER */
"#;
            
            // Add MSVC compatibility code after stdbit.h include
            if let Some(include_pos) = modified.find("#include \"compat/stdbit/stdbit.h\"") {
                let after_include = include_pos + "#include \"compat/stdbit/stdbit.h\"".len();
                let next_line = modified[after_include..].find('\n').unwrap_or(0);
                let insert_pos = after_include + next_line + 1;
                
                modified = format!("{}{}{}", 
                    &modified[..insert_pos],
                    msvc_compat,
                    &modified[insert_pos..]
                );
            }
        }
        
        if modified != content {
            fs::write(&ffmpeg_dec_c_path, modified)?;
            println!("✓ ffmpeg_dec.c modified: replaced <stdbit.h> with compat version and added MSVC compatibility");
        } else {
            println!("⚠ Could not find <stdbit.h> in ffmpeg_dec.c, skipping modification");
        }
        
        Ok(())
    }
    
    /// Get addon_src directory
    pub fn get_addon_src_dir(&self) -> &Path {
        &self.addon_src_dir
    }
}

