import { createRouter, createWebHistory } from 'vue-router'
import Greet from "../components/Greet.vue";

const routes = [
    {
        path: '/',
        name: 'Home',
        component: Greet
    }
]

const router = createRouter({
    history: createWebHistory(),
    routes
})

export default router