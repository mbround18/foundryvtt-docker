export const state = () => ({
    status: 'none',
    statusMessages: {
        none: 'You should not see this.',
        pending: 'We are waiting for FoundryVTT to initialize...',
        success: 'Yay! FoundryVTT is online! Redirecting you now!',
        failure: 'Fuck shit! Something blew up! check logs...'
    },
    alertActive: false
})

export const mutations = {
    showAlert(state, status) {
        state.alertActive = true
        state.status = status
    },
    hideAlert(state) {
        state.alertActive = false
    },
}

// export default new Vuex.Store({state, mutations})
