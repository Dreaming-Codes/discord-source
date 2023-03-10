<script lang="ts" setup>
import {useColorMode} from "@vueuse/core";

const mode = useColorMode();

function toggleMode() {
  mode.value = mode.value == 'dark' ? 'light' : 'dark';
}
</script>


<template>
  <v-navigation-drawer expand-on-hover rail>
    <v-list density="compact" nav>
      <v-list-item to="/home" prepend-icon="md:home" title="Home"></v-list-item>
      <v-list-item to="/settings" prepend-icon="md:settings" title="Settings"></v-list-item>
    </v-list>

    <template v-slot:append>
      <v-list density="compact" nav>
        <v-list-item :prepend-icon="mode === 'light' ? 'md:light_mode' : 'md:dark_mode'" :title="`${mode.charAt(0).toUpperCase() + mode.substr(1)} theme`" @click="toggleMode"></v-list-item>
      </v-list>
    </template>
  </v-navigation-drawer>
</template>