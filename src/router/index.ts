// Composables
import { createRouter, createWebHistory } from 'vue-router'
import Default from "../layouts/default/Default.vue";
import Home from "../views/Home.vue";
import Settings from "../views/Settings.vue";

const routes = [
    {
        path: '/',
        component: Default,
        children: [
            {
                path: 'home',
                name: 'Home',
                component: Home,
            },
            {
                path: 'settings',
                name: 'Settings',
                component: Settings,
            },
            {
                path: '',
                redirect: { path: 'home' },
            }
        ],
    },
]

const router = createRouter({
    history: createWebHistory(),
    routes,
})

export default router
