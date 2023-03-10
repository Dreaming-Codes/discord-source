/**
 * plugins/vuetify.ts
 *
 * Framework documentation: https://vuetifyjs.com`
 */

// Styles
import '@mdi/font/css/materialdesignicons.css'
import 'vuetify/styles'

// Composables
import {createVuetify} from 'vuetify';
import {useColorMode} from "@vueuse/core";
import 'material-design-icons-iconfont/dist/material-design-icons.css';
import { md } from 'vuetify/iconsets/md';
import { aliases, mdi} from "vuetify/iconsets/mdi";

const mode = useColorMode()

// https://vuetifyjs.com/en/introduction/why-vuetify/#feature-guides
export default createVuetify({
    theme: {
        defaultTheme: mode.value
    },
    icons: {
        defaultSet: 'mdi',
        aliases,
        sets: {
            md,
            mdi
        },
    }
})
