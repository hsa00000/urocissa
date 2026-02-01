<template>
  <v-col cols="12">
    <v-card border flat>
      <v-card-title class="font-weight-bold">Change Password</v-card-title>
      <v-divider thickness="4" variant="double"></v-divider>

      <v-list-item
        @click="enabled = !enabled"
        title="Enable Password Protection"
        subtitle="Turning this off makes your album public."
      >
        <template #append>
          <v-switch v-model="enabled" @click.stop color="primary" hide-details inset></v-switch>
        </template>
      </v-list-item>

      <v-divider></v-divider>
      <v-list-item>
        <v-text-field
          class="pt-2"
          v-model="oldPassword"
          prepend-icon="mdi-lock-outline"
          label="Current Password"
          :type="showOldPassword ? 'text' : 'password'"
          :append-inner-icon="showOldPassword ? 'mdi-eye' : 'mdi-eye-off'"
          variant="outlined"
          :placeholder="oldPasswordPlaceholder"
          :rules="[rules.requiredIfAction]"
          :disabled="!canInputOldPassword"
          persistent-placeholder
          density="compact"
          @click:append-inner="showOldPassword = !showOldPassword"
        ></v-text-field>
      </v-list-item>
      <v-list-item>
        <v-text-field
          class="pt-2"
          prepend-icon="mdi-lock"
          v-model="newPassword"
          label="New Password"
          :type="showNewPassword ? 'text' : 'password'"
          :append-inner-icon="showNewPassword ? 'mdi-eye' : 'mdi-eye-off'"
          variant="outlined"
          :rules="[rules.requiredIfAction, rules.noLeadingTrailingSpaces]"
          :disabled="!enabled"
          persistent-placeholder
          density="compact"
          @click:append-inner="showNewPassword = !showNewPassword"
        ></v-text-field
      ></v-list-item>
      <v-card-actions class="justify-end px-4 pb-4">
        <v-btn
          color="primary"
          variant="flat"
          :loading="loading"
          :disabled="!isValidAction"
          @click="savePassword"
          class="text-none"
        >
          Save Changes
        </v-btn>
      </v-card-actions>
    </v-card>
  </v-col>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { updatePassword, getConfig } from '@/api/config'
import { useMessageStore } from '@/store/messageStore'
import { useConfigStore } from '@/store/configStore'
import { tryWithMessageStore } from '@/script/utils/try_catch'

const hasPassword = defineModel<boolean>('hasPassword', { required: true })

const messageStore = useMessageStore('mainId')
const configStore = useConfigStore('mainId')

// --- State ---
const enabled = ref(true)
const oldPassword = ref('')
const newPassword = ref('')
const loading = ref(false)

const showOldPassword = ref(false)
const showNewPassword = ref(false)

// --- Computed: Status Checks ---
const hasExistingPassword = computed(() => hasPassword.value)

// Logic: Users can only input old password if one exists on the server.
const canInputOldPassword = computed(() => hasExistingPassword.value)

// User Intent
const isDisabling = computed(() => !enabled.value)
const isUpdating = computed(() => enabled.value && !!newPassword.value)

// Rule Logic
const isOldPasswordRequired = computed(() => {
  if (!hasExistingPassword.value) return false
  return isDisabling.value || isUpdating.value
})

const isValidAction = computed(() => {
  if (isOldPasswordRequired.value && !oldPassword.value) return false
  if (enabled.value && !newPassword.value) return false
  return true
})

// UI Helpers
const oldPasswordPlaceholder = computed(() => {
  if (!hasExistingPassword.value) return 'Not required'
  return isDisabling.value ? 'Required to disable password' : 'Required to verify identity'
})

// --- Watchers & Lifecycle ---
watch(
  hasPassword,
  (val) => {
    enabled.value = val
  },
  { immediate: true }
)

watch(enabled, (val) => {
  if (!val) {
    newPassword.value = ''
  }
})

// --- Validation Rules ---
const rules = {
  requiredIfAction: (v: string) => {
    return isOldPasswordRequired.value
      ? !!v || 'Current password is required to save changes'
      : true
  },
  noLeadingTrailingSpaces: (v: string) => {
    // Optional chaining safeguard
    if (!v) return true
    return v === v.trim() || 'Do not use spaces at the beginning or end of the password.'
  }
}

// --- Actions ---
const savePassword = async () => {
  if (!isValidAction.value) return

  loading.value = true

  await tryWithMessageStore('mainId', async () => {
    const finalNewPassword = isDisabling.value ? '' : newPassword.value.trim()

    await updatePassword(oldPassword.value, finalNewPassword)

    const newConfig = await getConfig()
    configStore.config = newConfig

    messageStore.success(
      isDisabling.value ? 'Password disabled successfully' : 'Password updated successfully'
    )

    // Reset local form state
    oldPassword.value = ''
    newPassword.value = ''
    return true
  })

  loading.value = false
}
</script>
