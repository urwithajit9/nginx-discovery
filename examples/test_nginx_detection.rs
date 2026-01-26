//! Test NGINX detection and system module
//!
//! Run with: cargo run --example test_nginx_detection --features system

#[cfg(feature = "system")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use nginx_discovery::system;

    println!("🔍 Testing NGINX Detection\n");
    println!("{}", "=".repeat(60));

    // Test 1: Find nginx binary
    println!("\n1️⃣  Finding nginx binary...");
    match system::find_nginx() {
        Ok(path) => {
            println!("   ✅ Found nginx at: {}", path.display());
        }
        Err(e) => {
            println!("   ❌ nginx not found: {}", e);
            println!("   💡 Install nginx: sudo apt-get install nginx (Ubuntu/Debian)");
            println!("                      brew install nginx (macOS)");
            return Ok(());
        }
    }

    // Test 2: Get nginx version
    println!("\n2️⃣  Getting nginx version...");
    match system::nginx_version() {
        Ok(version) => {
            println!("   ✅ {}", version);
        }
        Err(e) => {
            println!("   ❌ Failed: {}", e);
        }
    }

    // Test 3: Test config syntax
    println!("\n3️⃣  Testing nginx configuration syntax...");
    match system::test_config() {
        Ok(output) => {
            println!("   ✅ Configuration test passed");
            if !output.is_empty() {
                println!("   Output: {}", output.lines().next().unwrap_or(""));
            }
        }
        Err(e) => {
            println!("   ⚠️  Test failed (this is normal if nginx isn't configured)");
            println!("   Error: {}", e);
        }
    }

    // Test 4: Dump config (requires root/sudo)
    println!("\n4️⃣  Dumping nginx configuration...");
    println!("   ℹ️  This requires root/sudo permissions");
    match system::dump_config() {
        Ok(config) => {
            let lines: Vec<&str> = config.lines().collect();
            println!("   ✅ Successfully dumped config");
            println!("   📊 Config size: {} bytes", config.len());
            println!("   📄 Lines: {}", lines.len());
            println!("   Preview (first 5 lines):");
            for line in lines.iter().take(5) {
                println!("      {}", line);
            }
        }
        Err(e) => {
            println!("   ⚠️  Failed to dump config");
            println!("   Error: {}", e);
            println!("   💡 Try: sudo cargo run --example test_nginx_detection --features system");
        }
    }

    // Test 5: Full discovery
    println!("\n5️⃣  Testing full discovery...");
    match nginx_discovery::NginxDiscovery::from_running_instance() {
        Ok(discovery) => {
            println!("   ✅ Successfully created discovery from running instance");
            println!("\n   📊 Summary:");
            println!("{}", discovery.summary());

            println!("\n   🌐 Server names:");
            for name in discovery.server_names().iter().take(5) {
                println!("      - {}", name);
            }

            println!("\n   🔌 Listening ports:");
            for port in discovery.listening_ports() {
                println!("      - {}", port);
            }
        }
        Err(e) => {
            println!("   ⚠️  Discovery failed");
            println!("   Error: {}", e);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ Testing complete!\n");

    Ok(())
}

#[cfg(not(feature = "system"))]
fn main() {
    eprintln!("This example requires the 'system' feature.");
    eprintln!("Run with: cargo run --example test_nginx_detection --features system");
    std::process::exit(1);
}
