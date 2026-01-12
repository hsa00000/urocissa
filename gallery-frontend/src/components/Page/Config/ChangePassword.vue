<template>
  <v-col cols="12">
    <v-list-subheader class="font-weight-bold text-high-emphasis">Change Password</v-list-subheader>
    <v-card border flat class="rounded-lg">
      <v-card-text>
        <v-row dense>
          <v-col cols="12">
            <v-switch
              v-model="enabled"
              color="primary"
              label="Enable Password Protection"
              hide-details
              class="mb-4"
            ></v-switch>

            <v-text-field
              v-model="oldPassword"
              label="Current Password"
              :type="showOldPassword ? 'text' : 'password'"
              :append-inner-icon="showOldPassword ? 'mdi-eye' : 'mdi-eye-off'"
              prepend-icon="mdi-lock-check-outline"
              variant="outlined"
              density="comfortable"
              :placeholder="
                enabled ? 'Required if changing password' : 'Required to disable password'
              "
              :rules="[rules.requiredIfAction]"
              persistent-placeholder
              class="mb-3"
              @click:append-inner="showOldPassword = !showOldPassword"
            ></v-text-field>

            <v-text-field
              v-model="password"
              label="New Password"
              :type="showPassword ? 'text' : 'password'"
              :append-inner-icon="showPassword ? 'mdi-eye' : 'mdi-eye-off'"
              prepend-icon="mdi-lock-outline"
              variant="outlined"
              density="comfortable"
              persistent-placeholder
              hide-details="auto"
              :disabled="!enabled"
              class="mb-3"
              @click:append-inner="showPassword = !showPassword"
            ></v-text-field>

            <v-row justify="end" class="mt-2">
              <v-col cols="auto">
                <v-btn
                  color="primary"
                  variant="flat"
                  :loading="loading"
                  :disabled="!isValidAction"
                  @click="savePassword"
                  class="text-none"
                >
                  Update Password
                </v-btn>
              </v-col>
            </v-row>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>
  </v-col>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { updatePassword } from '@/api/config'
import { useMessageStore } from '@/store/messageStore'
import { tryWithMessageStore } from '@/script/utils/try_catch'

const messageStore = useMessageStore('mainId')
const password = ref('')
const oldPassword = ref('')
const enabled = ref(true)
const loading = ref(false)

const showPassword = ref(false)
const showOldPassword = ref(false)

// Reset fields when enabled changes
watch(enabled, (val) => {
  if (!val) {
    password.value = ''
    oldPassword.value = ''
  }
})

const isValidAction = computed(() => {
  // Valid if:
  // 1. Disable: oldPassword required
  // 2. Enable/Update: if new password typed, oldPassword required
  if (!enabled.value) {
    return !!oldPassword.value
  }
  if (password.value) {
    return !!oldPassword.value
  }
  return false // If enabled but no new password, nothing to update (unless we want to enforce re-enabling?)
  // But currently backend doesn't support "re-enable without changing"?
  // Actually if password is None in backend, and we send Some("pwd"), it sets it.
  // If we send Some(""), it removes it.
})

const rules = {
  requiredIfAction: (v: string) => {
    if (!enabled.value) return !!v || 'Required to disable password'
    if (password.value) return !!v || 'Required to change password'
    return true
  }
}

const savePassword = async () => {
  if (!isValidAction.value) return

  loading.value = true
  const success = await tryWithMessageStore('mainId', async () => {
    // If disabled, we send empty string to remove password (as per backend logic we decided: Some("") -> None)
    // Wait, backend logic:
    // if let Some(pwd) = req_data.password { if pwd.trim().is_empty() { None } ... }
    // So sending "" works for disabling.

    let newPwd = undefined

    if (!enabled.value) {
      newPwd = ''
    } else {
      // If enabled, we only send if user typed something (to update)
      if (password.value) {
        newPwd = password.value
      } else {
        // User didn't type new password, do nothing?
        return true
      }
    }

    await updatePassword(oldPassword.value, newPwd)

    // Reset fields
    oldPassword.value = ''
    password.value = ''
    return true
  })

  if (success === true) {
    messageStore.success(
      enabled.value ? 'Password updated successfully' : 'Password disabled successfully'
    )
  }

  loading.value = false
}
</script>
