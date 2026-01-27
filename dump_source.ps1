$dumpDir = "source_dump"
if (!(Test-Path $dumpDir)) { New-Item -ItemType Directory -Path $dumpDir | Out-Null }

# Force PowerShell to use UTF-8 for reading and writing
$OutputEncoding = [System.Text.Encoding]::UTF8

$groups = @{
    "root_files" = @(".", $false);
    "examples" = @("examples", $true);
    "tests" = @("tests", $true);
    "semantics" = @("semantics", $true);
    "src_root" = @("src", $false);
    "src_bin" = @("src\bin", $true);
    "src_kensho_smt" = @("src\kensho_smt", $true);
    "src_decompiler_root" = @("src\decompiler_prototype", $false);
    "src_decompiler_x86_64" = @("src\decompiler_prototype\x86_64", $true);
    "src_decompiler_arm64" = @("src\decompiler_prototype\arm64", $true);
    "src_decompiler_lifter" = @("src\decompiler_prototype\lifter", $true);
    "src_decompiler_optimizer" = @("src\decompiler_prototype\optimizer", $true);
    "src_decompiler_ssa" = @("src\decompiler_prototype\ssa", $true);
    "src_decompiler_mba" = @("src\decompiler_prototype\mba", $true);
    "src_decompiler_flattening" = @("src\decompiler_prototype\flattening", $true);
    "src_decompiler_vm_detection" = @("src\decompiler_prototype\vm_detection", $true);
    "src_decompiler_smt" = @("src\decompiler_prototype\smt", $true);
    "src_decompiler_z3_solver" = @("src\decompiler_prototype\z3_solver", $true);
    "src_decompiler_binary_loader" = @("src\decompiler_prototype\binary_loader", $true);
    "src_decompiler_debug_symbols" = @("src\decompiler_prototype\debug_symbols", $true);
}

foreach ($groupName in $groups.Keys) {
    $path = $groups[$groupName][0]
    $recurse = $groups[$groupName][1]
    $outFile = Join-Path $dumpDir "$groupName.rs.txt"
    
    if (Test-Path $outFile) { Remove-Item $outFile }
    
    if (Test-Path $path) {
        if ($recurse) {
            $files = Get-ChildItem -Path $path -Filter "*.rs" -Recurse
        } else {
            $files = Get-ChildItem -Path $path -Filter "*.rs"
        }

        $count = 0
        foreach ($file in $files) {
            # Skip if file is in a subdirectory when we want non-recursive
            if (-not $recurse -and $file.DirectoryName -ne (Resolve-Path $path).Path) { continue }

            "// ========================================================" | Out-File -FilePath $outFile -Append -Encoding utf8
            "// File: $($file.FullName.Substring((Get-Location).Path.Length + 1))" | Out-File -FilePath $outFile -Append -Encoding utf8
            "// ========================================================" | Out-File -FilePath $outFile -Append -Encoding utf8
            
            # Read explicitly as UTF-8
            Get-Content -Path $file.FullName -Encoding utf8 | Out-File -FilePath $outFile -Append -Encoding utf8
            "`n" | Out-File -FilePath $outFile -Append -Encoding utf8
            $count++
        }
        if ($count -gt 0) { Write-Host "Created $outFile ($count files)" }
    }
}
