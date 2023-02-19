<template>
  <HelloWorld />
</template>

<script lang="ts" setup>
import HelloWorld from '@/components/HelloWorld.vue'
import {invoke} from "@tauri-apps/api/tauri";

invoke('get_bd_path').then((res) => {
  console.log(res)
})
</script>
