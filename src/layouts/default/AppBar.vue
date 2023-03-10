<script lang="ts" setup>
import {useColorMode} from "@vueuse/core";

const mode = useColorMode();

function toggleMode() {
  mode.value = mode.value == 'dark' ? 'light' : 'dark';
}
</script>


<template>
  <v-app-bar flat>
    <v-app-bar-title>
      <v-icon icon="mdi-circle-slice-4" />
      Base Preset
    </v-app-bar-title>

    <template v-slot:append>
      <v-btn @click="toggleMode" :icon="mode === 'light' ? 'md:light_mode' : 'md:dark_mode'"></v-btn>
    </template>
  </v-app-bar>
</template>