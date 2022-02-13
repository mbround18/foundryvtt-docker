import path from "path"
import { defineNuxtConfig } from 'nuxt3'
// https://v3.nuxtjs.org/docs/directory-structure/nuxt.config
export default defineNuxtConfig({
    typescript: {
        shim: false
    },
    target: 'static',
    ssr: false,
    buildDir: path.join(__dirname, "./dist/"),
    css: ['vuetify/lib/styles/main.sass'],
    build: {
        transpile: ['vuetify']
    },
    vite: {
        define: {
            'process.env.DEBUG': 'false',
        }
    }
})

