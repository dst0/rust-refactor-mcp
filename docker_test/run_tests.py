#!/usr/bin/env python3
import os
import sys
import subprocess
import re
import shutil
from pathlib import Path

REPOS = [
    "https://github.com/tokio-rs/tokio",
    "https://github.com/hyperium/hyper",
    "https://github.com/hyperium/axum",
    "https://github.com/serde-rs/serde",
    "https://github.com/clap-rs/clap",
    "https://github.com/rustls/rustls",
    "https://github.com/rust-random/rand",
    "https://github.com/rust-lang/log",
    "https://github.com/bitflags/bitflags",
    "https://github.com/dtolnay/thiserror",
    "https://github.com/dtolnay/syn",
    "https://github.com/dtolnay/quote",
    "https://github.com/rust-lang/once_cell",
    "https://github.com/crossbeam-rs/crossbeam",
    "https://github.com/rayon-rs/rayon",
    "https://github.com/sfackler/rust-openssl",
    "https://github.com/alexcrichton/git2-rs",
    "https://github.com/seanmonstar/reqwest",
    "https://github.com/BurntSushi/ripgrep",
    "https://github.com/chronotope/chrono",
    "https://github.com/astral-sh/ruff",
    "https://github.com/bevyengine/bevy",
    "https://github.com/ratatui/ratatui"
]

TOOL = "/rust-refactor-mcp/target/release/rust-refactor-mcp"

# Statistics structure
stats = {
    repo: {
        "baseline": "SKIP",
        "ANALYZE_DEPS": "SKIP",
        "FIND_DEAD_CODE": "SKIP",
        "EXTRACT": "SKIP",
        "RENAME": "SKIP",
        "FORMAT": "SKIP",
        "OPTIMIZE_IMPORTS": "SKIP",
        "SSR": "SKIP",
        "EXPAND": "SKIP",
        "SPLIT_DIR": "SKIP",
        "FIX_CARGO": "SKIP",
        "PREFLIGHT": "SKIP",
    } for repo in REPOS
}

crashes = 0
failures = 0

def run_cmd(args, cwd=None):
    try:
        res = subprocess.run(args, capture_output=True, text=True, cwd=cwd)
        # Check for panics in stderr/stdout
        if "panicked at" in res.stderr or "panicked at" in res.stdout or "thread 'main' panicked" in res.stderr:
            return "CRASH", res.stdout + "\n" + res.stderr
        if res.returncode != 0:
            return "FAIL", res.stdout + "\n" + res.stderr
        return "PASS", res.stdout
    except Exception as e:
        return "CRASH", str(e)

for repo_url in REPOS:
    repo_name = repo_url.split("/")[-1]
    print(f"\n======================================")
    print(f"Testing {repo_name}")
    print(f"======================================")

    # Clone
    repo_dir = Path(f"/test/{repo_name}")
    if repo_dir.exists():
        try:
            shutil.rmtree(repo_dir)
        except Exception:
            pass
    
    print(f"Cloning {repo_url}...")
    status, out = run_cmd(["git", "clone", "--depth", "1", repo_url, str(repo_dir)])
    if status != "PASS":
        print(f"Failed to clone {repo_name}: {out}")
        continue

    # Baseline check
    print("Running Baseline Check (cargo check)...")
    status, out = run_cmd(["cargo", "check"], cwd=repo_dir)
    stats[repo_url]["baseline"] = status
    if status != "PASS":
        print(f"Baseline cargo check failed for {repo_name}. Skipping further tests.")
        continue

    # ANALYZE_DEPS
    print("Running ANALYZE_DEPS...")
    status, out = run_cmd([TOOL, ".", "ANALYZE_DEPS", "."], cwd=repo_dir)
    stats[repo_url]["ANALYZE_DEPS"] = status
    if status == "CRASH":
        crashes += 1
        print("CRASHED!")

    # FIND_DEAD_CODE
    print("Running FIND_DEAD_CODE...")
    status, out = run_cmd([TOOL, ".", "FIND_DEAD_CODE", "."], cwd=repo_dir)
    stats[repo_url]["FIND_DEAD_CODE"] = status
    if status == "CRASH":
        crashes += 1
        print("CRASHED!")

    # Find representative Rust files and entities
    rs_files = list(repo_dir.glob("src/**/*.rs"))
    entity_found = False
    
    for rs_file in rs_files[:5]:  # Check up to 5 files to find a valid entity
        try:
            content = rs_file.read_text(errors="ignore")
        except Exception:
            continue
        
        # Look for pub struct, pub enum, pub fn
        m_struct = re.search(r"\bpub\s+struct\s+([A-Za-z0-9_]+)", content)
        m_enum = re.search(r"\bpub\s+enum\s+([A-Za-z0-9_]+)", content)
        m_fn = re.search(r"\bpub\s+fn\s+([A-Za-z0-9_]+)", content)
        
        entity_name = None
        if m_struct:
            entity_name = m_struct.group(1)
        elif m_enum:
            entity_name = m_enum.group(1)
        elif m_fn:
            entity_name = m_fn.group(1)
            
        if entity_name:
            print(f"Found entity '{entity_name}' in {rs_file.relative_to(repo_dir)}")
            entity_found = True
            
            # EXTRACT (Single Entity Extraction)
            tmp_extract_dir = Path("/tmp/extracted_entities")
            if tmp_extract_dir.exists():
                try:
                    shutil.rmtree(tmp_extract_dir)
                except Exception:
                    pass
            tmp_extract_dir.mkdir(parents=True, exist_ok=True)
            
            print(f"Running EXTRACT on {entity_name}...")
            status, out = run_cmd([TOOL, str(rs_file), entity_name, str(tmp_extract_dir)], cwd=repo_dir)
            stats[repo_url]["EXTRACT"] = status
            if status == "CRASH":
                crashes += 1
                print("CRASHED!")
            
            # RENAME
            tmp_rename_file = Path("/tmp/test_rename.rs")
            if tmp_rename_file.exists():
                try:
                    tmp_rename_file.unlink()
                except Exception:
                    pass
            try:
                shutil.copy(rs_file, tmp_rename_file)
            except Exception:
                pass
            
            print(f"Running RENAME on {entity_name}...")
            status, out = run_cmd([TOOL, str(tmp_rename_file), "RENAME", entity_name, "RenamedEntity"], cwd=repo_dir)
            stats[repo_url]["RENAME"] = status
            if status == "CRASH":
                crashes += 1
                print("CRASHED!")

            # FORMAT
            tmp_format_file = Path("/tmp/test_format.rs")
            if tmp_format_file.exists():
                try:
                    tmp_format_file.unlink()
                except Exception:
                    pass
            try:
                shutil.copy(rs_file, tmp_format_file)
            except Exception:
                pass
            
            print("Running FORMAT...")
            status, out = run_cmd([TOOL, str(tmp_format_file), "FORMAT", str(tmp_format_file)], cwd=repo_dir)
            stats[repo_url]["FORMAT"] = status
            if status == "CRASH":
                crashes += 1
                print("CRASHED!")

            # OPTIMIZE_IMPORTS
            tmp_opt_file = Path("/tmp/test_optimize.rs")
            if tmp_opt_file.exists():
                try:
                    tmp_opt_file.unlink()
                except Exception:
                    pass
            try:
                shutil.copy(rs_file, tmp_opt_file)
            except Exception:
                pass
            
            print("Running OPTIMIZE_IMPORTS...")
            status, out = run_cmd([TOOL, str(tmp_opt_file), "OPTIMIZE_IMPORTS", str(tmp_opt_file)], cwd=repo_dir)
            stats[repo_url]["OPTIMIZE_IMPORTS"] = status
            if status == "CRASH":
                crashes += 1
                print("CRASHED!")

            # SSR
            tmp_ssr_file = Path("/tmp/test_ssr.rs")
            if tmp_ssr_file.exists():
                try:
                    tmp_ssr_file.unlink()
                except Exception:
                    pass
            try:
                shutil.copy(rs_file, tmp_ssr_file)
            except Exception:
                pass
            
            print("Running SSR...")
            status, out = run_cmd([TOOL, str(tmp_ssr_file), "SSR", entity_name, entity_name], cwd=repo_dir)
            stats[repo_url]["SSR"] = status
            if status == "CRASH":
                crashes += 1
                print("CRASHED!")

            # EXPAND
            print("Running EXPAND...")
            status, out = run_cmd([TOOL, ".", "EXPAND", str(rs_file)], cwd=repo_dir)
            stats[repo_url]["EXPAND"] = status
            if status == "CRASH":
                crashes += 1
                print("CRASHED!")

            break  # Only test one entity per repo
            
    if not entity_found:
        print("No suitable public entity found in src/ files. Skipping single-entity tests.")

    # SPLIT_DIR
    src_dir = repo_dir / "src"
    if src_dir.exists() and src_dir.is_dir():
        print("Running SPLIT_DIR...")
        status, out = run_cmd([TOOL, "SPLIT_DIR", "src"], cwd=repo_dir)
        stats[repo_url]["SPLIT_DIR"] = status
        if status == "CRASH":
            crashes += 1
            print("CRASHED!")

        # FIX_CARGO
        print("Running FIX_CARGO...")
        status, out = run_cmd([TOOL, ".", "FIX_CARGO", "Cargo.toml"], cwd=repo_dir)
        stats[repo_url]["FIX_CARGO"] = status
        if status == "CRASH":
            crashes += 1
            print("CRASHED!")

        # PREFLIGHT
        print("Running PREFLIGHT...")
        status, out = run_cmd([TOOL, ".", "PREFLIGHT", "Cargo.toml"], cwd=repo_dir)
        stats[repo_url]["PREFLIGHT"] = status
        if status == "CRASH":
            crashes += 1
            print("CRASHED!")
        elif status == "FAIL":
            failures += 1
            print("PREFLIGHT compilation failed (expected on complex codebases).")
        else:
            print("PREFLIGHT passed!")

# Print summary table
print("\n=========================================================================")
print("                             TEST SUMMARY")
print("=========================================================================")
print(f"{'Repository':<20} | {'ANALYZE':<7} | {'DEAD':<4} | {'EXTR':<4} | {'RENM':<4} | {'FMT':<4} | {'OPT':<4} | {'SSR':<4} | {'EXP':<4} | {'SPLT':<4} | {'FIX':<4} | {'PRE':<4}")
print("-" * 110)

for repo_url in REPOS:
    repo_name = repo_url.split("/")[-1]
    r_stats = stats[repo_url]
    print(f"{repo_name:<20} | "
          f"{r_stats['ANALYZE_DEPS']:<7} | "
          f"{r_stats['FIND_DEAD_CODE']:<4} | "
          f"{r_stats['EXTRACT']:<4} | "
          f"{r_stats['RENAME']:<4} | "
          f"{r_stats['FORMAT']:<4} | "
          f"{r_stats['OPTIMIZE_IMPORTS']:<4} | "
          f"{r_stats['SSR']:<4} | "
          f"{r_stats['EXPAND']:<4} | "
          f"{r_stats['SPLIT_DIR']:<4} | "
          f"{r_stats['FIX_CARGO']:<4} | "
          f"{r_stats['PREFLIGHT']:<4}")

print("=========================================================================")
print(f"Total Tool Crashes (Panics): {crashes}")
print(f"Total Post-Split Compile Failures (PREFLIGHT): {failures}")
print("=========================================================================")

sys.exit(crashes)  # Fail the test run if any tool panics/crashes
