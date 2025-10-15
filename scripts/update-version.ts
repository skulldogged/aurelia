#!/usr/bin/env node

import { execSync } from 'child_process'
import fs from 'fs'

const getGitCommitHash = (): string => {
  try {
    return execSync('git rev-parse --short HEAD').toString().trim()
  } catch {
    console.warn('Could not get git commit hash, using fallback')
    return 'unknown'
  }
}

const updateCargoToml = (version: string): void => {
  const cargoPath = 'src-tauri/Cargo.toml'
  let content = fs.readFileSync(cargoPath, 'utf8')

  // Update version in [package] section
  content = content.replace(/version = "[^"]*"/, `version = "${version}"`)

  fs.writeFileSync(cargoPath, content)
  console.log(`Updated ${cargoPath} with version: ${version}`)
}

const updateTauriConfig = (version: string): void => {
  const configPath = 'src-tauri/tauri.conf.json'
  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'))

  config.version = version

  fs.writeFileSync(configPath, JSON.stringify(config, null, 2))
  console.log(`Updated ${configPath} with version: ${version}`)
}

const main = (): void => {
  const commitHash = getGitCommitHash()
  const version = `unstable-${commitHash}`

  console.log(`Generating version: ${version}`)

  updateCargoToml(version)
  updateTauriConfig(version)

  console.log('Version update complete!')
}

main()