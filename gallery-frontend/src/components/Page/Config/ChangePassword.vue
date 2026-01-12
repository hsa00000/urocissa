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
              :placeholder="oldPasswordPlaceholder"
              :rules="[rules.requiredIfAction]"
              persistent-placeholder
              class="mb-3"
              @click:append-inner="showOldPassword = !showOldPassword"
              :disabled="!canInputOldPassword"
            ></v-text-field>

            <v-text-field
              v-model="newPassword"
              label="New Password"
              :type="showNewPassword ? 'text' : 'password'"
              :append-inner-icon="showNewPassword ? 'mdi-eye' : 'mdi-eye-off'"
              prepend-icon="mdi-lock-outline"
              variant="outlined"
              density="comfortable"
              persistent-placeholder
              hide-details="auto"
              :disabled="!enabled"
              class="mb-3"
              :rules="[rules.requiredIfAction, rules.noLeadingTrailingSpaces]"
              @click:append-inner="showNewPassword = !showNewPassword"
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
import { ref, watch, computed, onMounted } from 'vue'
import { updatePassword, getConfig } from '@/api/config'
import { useMessageStore } from '@/store/messageStore'
import { useConfigStore } from '@/store/configStore'
import { tryWithMessageStore } from '@/script/utils/try_catch'

const messageStore = useMessageStore('mainId')
const configStore = useConfigStore('mainId')

// --- State ---
const enabled = ref(true)
const oldPassword = ref('')
const newPassword = ref('') // Renamed from 'password' for clarity
const loading = ref(false)

const showOldPassword = ref(false)
const showNewPassword = ref(false) // Renamed from 'showPassword'

// --- Computed: Status Checks ---
// Check if password is currently set on the server side
const hasExistingPassword = computed(() => configStore.config?.hasPassword ?? false)

// SIMPLE LOGIC:
// You can only input an old password if one actually exists on the server.
// It doesn't matter if you are enabling or disabling; if it exists, you might need to type it.
const canInputOldPassword = computed(() => hasExistingPassword.value)

// Determine user intent
const isDisabling = computed(() => !enabled.value)
const isUpdating = computed(() => enabled.value && !!newPassword.value)

// Determine if old password is required
// Rule: If password exists and (disabling OR entering new password), old password is required
const isOldPasswordRequired = computed(() => {
  if (!hasExistingPassword.value) return false
  return isDisabling.value || isUpdating.value
})

// Whether the button is clickable
const isValidAction = computed(() => {
  // 1. If old password is required but not filled -> Invalid
  if (isOldPasswordRequired.value && !oldPassword.value) return false

  // 2. If in enable mode but new password not filled -> Invalid (prevent sending empty new password)
  if (enabled.value && !newPassword.value) return false

  return true
})

// UI Placeholder logic
const oldPasswordPlaceholder = computed(() => {
  if (!hasExistingPassword.value) return 'Not required'
  return isDisabling.value ? 'Required to disable password' : 'Required to verify identity'
})

// --- Watchers ---
onMounted(() => {
  if (configStore.config?.hasPassword !== undefined) {
    enabled.value = configStore.config.hasPassword
  }
})

// Sync local state with store config
watch(
  () => configStore.config?.hasPassword,
  (val) => {
    if (val !== undefined) enabled.value = val
  }
)

// Reset fields if user toggles switch off
watch(enabled, (val) => {
  if (!val) {
    newPassword.value = ''
  }
})

// --- Form Rules ---
const rules = {
  requiredIfAction: (v: string) => {
    return isOldPasswordRequired.value
      ? !!v || 'Current password is required to save changes'
      : true
  },
  noLeadingTrailingSpaces: (v: string) => {
    return v === v.trim() || 'Do not use spaces at the beginning or end of the password.'
  }
}

// --- Actions ---
const savePassword = async () => {
  if (!isValidAction.value) return

  loading.value = true

  await tryWithMessageStore('mainId', async () => {
    // Prepare Payload: If disabling, send empty string; if enabling/changing, send new password
    const finalNewPassword = isDisabling.value ? '' : newPassword.value.trim()

    // API Call
    await updatePassword(oldPassword.value, finalNewPassword)

    // Update Local Store manually to reflect change immediately
    const newConfig = await getConfig()
    configStore.config = newConfig

    // Success Handling
    messageStore.success(
      isDisabling.value ? 'Password disabled successfully' : 'Password updated successfully'
    )

    // Cleanup
    oldPassword.value = ''
    newPassword.value = ''
    return true
  })

  loading.value = false
}
</script>
