package com.aurelia.app

import uniffi.aurelia_core.ping

object CoreBridge {
  fun ping(): String {
    return try {
      ping()
    } catch (error: UnsatisfiedLinkError) {
      "core not loaded"
    } catch (error: RuntimeException) {
      "core error"
    }
  }
}
