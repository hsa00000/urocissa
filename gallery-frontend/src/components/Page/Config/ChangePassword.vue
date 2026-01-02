<template>
  <v-col cols="12">
    <v-list-subheader class="font-weight-bold text-high-emphasis">Change Password</v-list-subheader>
    <v-card border flat class="rounded-lg">
      <v-card-text>
        <v-row dense>
          <v-col cols="12">
            <v-text-field
              v-model="oldPassword"
              label="Current Password"
              :type="showOldPassword ? 'text' : 'password'"
              :append-inner-icon="showOldPassword ? 'mdi-eye' : 'mdi-eye-off'"
              prepend-icon="mdi-lock-check-outline"
              variant="outlined"
              density="comfortable"
              placeholder="Required if changing password"
              :rules="[rules.requiredIfNewPassword]"
              persistent-placeholder
              hide-details="auto"
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
              placeholder="Leave empty to keep current"
              persistent-placeholder
              hide-details="auto"
              @click:append-inner="showPassword = !showPassword"
            ></v-text-field>
          </v-col>
        </v-row>
      </v-card-text>
    </v-card>
  </v-col>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const password = defineModel<string>('password', { required: true })
const oldPassword = defineModel<string>('oldPassword', { required: true })

const showPassword = ref(false)
const showOldPassword = ref(false)

const rules = {
  requiredIfNewPassword: (v: string) =>
    !password.value || !!v || 'Required to change password'
}
</script>
