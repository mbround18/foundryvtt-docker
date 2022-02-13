// @ts-ignore
import { defineNuxtPlugin } from '#app'
// @ts-ignore
import { createVuetify } from 'vuetify'

// Import everything
import * as components from 'vuetify/components'

export default defineNuxtPlugin((nuxtApp) => {
    const vuetify = createVuetify({
        components
    })
    nuxtApp.vueApp.use(vuetify)
})
