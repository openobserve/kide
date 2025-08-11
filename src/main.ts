import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './style.css'
import App from './App.vue'
import errorHandlerPlugin from './plugins/errorHandler'

// Set dark theme on document element permanently
document.documentElement.classList.add('dark')

const app = createApp(App)
const pinia = createPinia()

// Install plugins
app.use(pinia)
app.use(errorHandlerPlugin)

app.mount('#app')