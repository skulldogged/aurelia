package com.aurelia.app.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.aurelia.app.storage.SessionStore

@Composable
fun LoginScreen(sessionStore: SessionStore) {
  val viewModel: LoginViewModel = viewModel(
    factory = LoginViewModelFactory(sessionStore)
  )
  val state by viewModel.state.collectAsState()

  LaunchedEffect(state.token) {
    if (state.token != null) {
      // The parent observes session changes and will swap screens.
    }
  }

  Column(
    modifier = Modifier
      .fillMaxSize()
      .padding(PaddingValues(24.dp)),
    verticalArrangement = Arrangement.spacedBy(16.dp)
  ) {
    Text(text = "Sign in", style = MaterialTheme.typography.headlineMedium)

    OutlinedTextField(
      value = state.serverUrl,
      onValueChange = viewModel::updateServerUrl,
      label = { Text("Server URL") },
      placeholder = { Text("https://your-jellyfin") },
      singleLine = true,
      modifier = Modifier.fillMaxWidth()
    )

    OutlinedTextField(
      value = state.username,
      onValueChange = viewModel::updateUsername,
      label = { Text("Username") },
      singleLine = true,
      modifier = Modifier.fillMaxWidth()
    )

    OutlinedTextField(
      value = state.password,
      onValueChange = viewModel::updatePassword,
      label = { Text("Password") },
      visualTransformation = PasswordVisualTransformation(),
      singleLine = true,
      modifier = Modifier.fillMaxWidth()
    )

    if (state.error != null) {
      Text(text = state.error ?: "", color = MaterialTheme.colorScheme.error)
    }

    Button(
      onClick = viewModel::submit,
      enabled = !state.isSubmitting,
    ) {
      Text(text = if (state.isSubmitting) "Signing in..." else "Sign in")
    }

    if (state.token != null) {
      Text(text = "Signed in")
    }
  }
}
