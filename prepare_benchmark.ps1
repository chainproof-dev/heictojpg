$ErrorActionPreference = "Stop"

$BenchmarkDir = Join-Path $PSScriptRoot "static\benchmark"
$ManifestFile = Join-Path $BenchmarkDir "manifest.json"

# Create directory
if (!(Test-Path $BenchmarkDir)) {
    New-Item -ItemType Directory -Path $BenchmarkDir | Out-Null
    Write-Host "Created benchmark directory: $BenchmarkDir"
}

$Sources = @(
    "https://freetestdata.com/wp-content/uploads/2024/05/FTD_1.3.heic",
    "https://freetestdata.com/wp-content/uploads/2024/05/FTD_5.1MB.heic",
    "https://heic.digital/download-sample/shelf-christmas-decoration.heic",
    "https://heic.digital/download-sample/sewing-threads.heic",
    "https://heic.digital/download-sample/chef-with-trumpet.heic",
    "https://heic.digital/download-sample/childrens-show-theater.heic",
    "https://heic.digital/download-sample/soundboard.heic"
)

$Files = @()
$Index = 1

Write-Host "Downloading sample images..."

foreach ($Url in $Sources) {
    $FileName = "sample_$Index.heic"
    $OutputPath = Join-Path $BenchmarkDir $FileName
    
    if (!(Test-Path $OutputPath)) {
        Write-Host "Downloading $Url to $FileName..."
        try {
            Invoke-WebRequest -Uri $Url -OutFile $OutputPath
        } catch {
            Write-Warning "Failed to download $Url"
            continue
        }
    } else {
        Write-Host "$FileName already exists."
    }
    
    $FileInfo = Get-Item $OutputPath
    $Files += @{
        name = $FileName
        size = $FileInfo.Length
        path = "benchmark/$FileName"
    }
    $Index++
}

# Duplicate files to create a larger dataset (target ~21 files)
Write-Host "Duplicating files to create larger dataset..."
$OriginalCount = $Files.Count
for ($i = 0; $i -lt 2; $i++) {
    for ($j = 0; $j -lt $OriginalCount; $j++) {
        $SourceFile = $Files[$j]
        $NewIndex = $Index + ($i * $OriginalCount) + $j
        $NewName = "sample_copy_$NewIndex.heic"
        $NewPath = Join-Path $BenchmarkDir $NewName
        $SourcePath = Join-Path $BenchmarkDir $SourceFile.name
        
        if (Test-Path $SourcePath) {
             Copy-Item -Path $SourcePath -Destination $NewPath -Force
             $Files += @{
                name = $NewName
                size = $SourceFile.size
                path = "benchmark/$NewName"
            }
        }
    }
}

# Generate Manifest
$ManifestData = @{
    files = $Files
    total_files = $Files.Count
    total_size = ($Files | Measure-Object -Property size -Sum).Sum
}

$ManifestData | ConvertTo-Json -Depth 3 | Set-Content $ManifestFile
Write-Host "Manifest generated at $ManifestFile with $($Files.Count) files."
