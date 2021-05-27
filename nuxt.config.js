export default {
    target: 'static',
    buildModules: [
        ['@nuxtjs/vuetify', {dark: true}]
    ],
    dir: {
        // Rename `pages` directory to `routes`
        pages: 'src/client/pages',
        components: 'src/client/components',
        assets: 'src/client/assets',
        static: 'src/client/static',
        layouts: 'src/client/layouts',
        store: 'src/client/store'
    },
    generate: {
        dir: 'dist/client'
    }
}
