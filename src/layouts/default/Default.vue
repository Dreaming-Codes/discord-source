<template>
  <v-app>
    <default-bar />

    <default-view />

    <license-check />
  </v-app>
</template>

<script lang="ts" setup>
  import DefaultBar from './AppBar.vue'
  import DefaultView from './View.vue'
  import LicenseCheck from "./LicenseCheck.vue";
</script>
