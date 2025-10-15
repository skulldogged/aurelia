#!/usr/bin/env node

import { execSync } from 'child_process'
import fs from 'fs'

const getLatestCommitHash = (): string => {
  try {
    return execSync('git rev-parse HEAD').toString().trim()
  } catch {
    console.warn('Could not get latest commit hash')
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
  const commitHash = getLatestCommitHash()
  const version = `0.1.0-unstable.${commitHash.substring(0, 8)}`

  console.log(`Updating version to: ${version}`)

  updateCargoToml(version)
  updateTauriConfig(version)

  console.log('Version update complete!')
}

main()