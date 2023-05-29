<script setup lang="ts">
import {ref} from "vue";

const dialog = ref(false);
</script>

<template>
    <div class="text-center">
        <v-btn
                color="primary"
        >
            New target

            <v-dialog
                    v-model="dialog"
                    activator="parent"
                    width="auto"
            >
                <v-card>
                    <v-card-text>
                        To add the first target you need to manually add it to the scene. To do this, add the following link as browser source to your scene: http://localhost:4651/name you can replace name with any name you want. This will create a new target with the name you specified.<br>
                        <span class="text-red">Remember to remove any default custom CSS.</span><br>
                        For width and height you can use any value you want, but it is recommended to use 1920x1080 since most discord webcam and streams are 1920x1080.<br>
                        For fps you should use 30 since discord webcam are 30 fps.
                    </v-card-text>
                    <v-card-actions>
                        <v-btn color="primary" block @click="dialog = false">Okay</v-btn>
                    </v-card-actions>
                </v-card>
            </v-dialog>
        </v-btn>
    </div>
</template>