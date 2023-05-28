<script setup lang="ts">
import {computed, ref} from "vue";
import {invoke} from "@tauri-apps/api/tauri";

enum LicenseStatus {
  Loading = 0,
  Error = 1,
  Valid = 2,
}

const dialog = ref(LicenseStatus.Loading);

const shouldDisplayDialog = computed(() => {
  return dialog.value !== LicenseStatus.Valid;
});

invoke('check_license').then(() => {
  dialog.value = LicenseStatus.Valid;
}).catch(() => {
  dialog.value = LicenseStatus.Error;
});
</script>

<template>
  <v-row justify="center">
    <v-dialog
        v-model="shouldDisplayDialog"
        persistent
        width="auto"
    >
      <v-card class="rounded-lg">
        <v-progress-linear :indeterminate="dialog !== LicenseStatus.Error" model-value="100" :color="dialog === LicenseStatus.Error ? 'red' : 'yellow'"></v-progress-linear>
        <v-card-title class="text-h5 text-center">
          License Check
        </v-card-title>
        <v-card-text v-if="dialog === LicenseStatus.Loading">I'm checking your license, if you haven't purchased the software you can get a free trial! <a @click.prevent="invoke('open_ds_invite')" href="">Join my Discord</a> for more infos</v-card-text>
        <v-card-text v-else>License check failed, is your discord client started? <a @click.prevent="invoke('open_ds_invite')" href="">Join my Discord</a> for help!</v-card-text>
      </v-card>
    </v-dialog>
  </v-row>
</template>