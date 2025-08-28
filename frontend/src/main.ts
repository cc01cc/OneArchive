import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
// import './style.css';
import Aura from '@primeuix/themes/aura';

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
        // semantic: {
        //     primary: {
        //         50: '{stone.50}',
        //         100: '{stone.100}',
        //         200: '{stone.200}',
        //         300: '{stone.300}',
        //         400: '{stone.400}',
        //         500: '{stone.500}',
        //         600: '{stone.600}',
        //         700: '{stone.700}',
        //         800: '{stone.800}',
        //         900: '{stone.900}',
        //         950: '{stone.950}'
        //     },
        //     colorScheme: {
        //         light: {
        //             surface: {
        //                 0: '#ffffff',
        //                 50: '{neutral.50}',
        //                 100: '{neutral.100}',
        //                 200: '{neutral.200}',
        //                 300: '{neutral.300}',
        //                 400: '{neutral.400}',
        //                 500: '{neutral.500}',
        //                 600: '{neutral.600}',
        //                 700: '{neutral.700}',
        //                 800: '{neutral.800}',
        //                 900: '{neutral.900}',
        //                 950: '{neutral.950}'
        //             },
        //             formField: {
        //                 hoverBorderColor: '{primary.color}',
        //             }
        //         },
        //         dark: {
        //             surface: {
        //                 0: '#000000',
        //                 50: '{neutral.50}',
        //                 100: '{neutral.100}',
        //                 200: '{neutral.200}',
        //                 300: '{neutral.300}',
        //                 400: '{neutral.400}',
        //                 500: '{neutral.500}',
        //                 600: '{neutral.600}',
        //                 700: '{neutral.700}',
        //                 800: '{neutral.800}',
        //                 900: '{neutral.900}',
        //                 950: '{neutral.950}'
        //             }
        //         }
        //     }
        // }
    }
});
app.use(ToastService);
app.use(router)

app.mount('#app')