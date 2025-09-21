/*
 *   Copyright (c) 2025 Zheng, Yihong (ZEO, github.com/cc01cc)
 *   All rights reserved.

 *   Licensed under the Apache License, Version 2.0 (the "License");
 *   you may not use this file except in compliance with the License.
 *   You may obtain a copy of the License at

 *   http://www.apache.org/licenses/LICENSE-2.0

 *   Unless required by applicable law or agreed to in writing, software
 *   distributed under the License is distributed on an "AS IS" BASIS,
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *   See the License for the specific language governing permissions and
 *   limitations under the License.
 */

import { createRouter, createWebHistory } from 'vue-router'
import DefaultLayout from '../layouts/DefaultLayout.vue'
import ConfigPage from '../pages/Config.vue'

const routes = [
    {
        path: '/',
        redirect: '/task-center'
    },

    {
        path: '/config',
        name: 'Config',
        component: ConfigPage
    },
    {
        path: '/',
        component: DefaultLayout,
        children: [
            {
                path: 'task-center',
                name: 'TaskCenter',
                component: () => import('../pages/TaskCenter.vue')
            },
            {
                path: 'scan',
                name: 'Scan',
                component: () => import('../pages/Scan.vue')
            },
            {
                path: 'resource-manager',
                name: 'ResourceManager',
                component: () => import('../pages/ResourceManager.vue')
            },
            {
                path: '/workspace',
                name: 'Workspace',
                component: () => import('../pages/Workspace.vue')
            }
        ]
    }
]


const router = createRouter({
    history: createWebHistory(),
    routes
})

export default router