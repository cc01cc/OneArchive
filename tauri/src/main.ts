import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import './style.css';
import Aura from '@primeuix/themes/aura';
import 'primeicons/primeicons.css';
import { createPinia } from 'pinia'
import ConfirmationService from 'primevue/confirmationservice';
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

// PrimeVue 相关导入
import { definePreset } from '@primeuix/themes';
import PrimeVue from 'primevue/config';
import ToastService from 'primevue/toastservice';

const app = createApp(App)

const MyPreset = definePreset(Aura, {
    semantic: {
        primary: {
            50: '{neutral.50}',
            100: '{neutral.100}',
            200: '{neutral.200}',
            300: '{neutral.300}',
            400: '{neutral.400}',
            500: '{neutral.500}',
            600: '{neutral.600}',
            700: '{neutral.700}',
            800: '{neutral.800}',
            900: '{neutral.900}',
            950: '{neutral.950}'
        }
    }
});
// 使用 PrimeVue
app.use(PrimeVue, {
    theme: {
        preset: MyPreset,
        options: {
            darkModeSelector: false || 'none',
        },
    }
});

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

app.use(ToastService);
app.use(ConfirmationService);
app.use(router)
app.use(pinia)

app.mount('#app')

