<template>
  <router-view />
</template>

<script lang="ts" setup>
import {invoke} from "@tauri-apps/api/tauri";
import {useColorMode} from "@vueuse/core";
import {useTheme} from "vuetify";
import {watch} from "vue";

const mode = useColorMode()
const theme = useTheme()

watch(mode, (value) => {
  theme.global.name.value = value;
})

invoke("get_config").then(console.log)
</script>
