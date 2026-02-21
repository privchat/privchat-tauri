import {createWebHistory, createRouter} from 'vue-router';
import Home from '../views/Home.vue';
import Login from '../views/Login.vue';
import ChatView from '../views/ChatView.vue';
import PeopleView from '../views/PeopleView.vue';
import SettingsView from '../views/SettingsView.vue';
import AccountView from '../views/AccountView.vue';

const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: '/',
            name: 'login',
            component: Login
        },
        {
            path: '/home',
            name: 'home',
            component: Home,
            children: [
                {
                    path: '/chat',
                    name: 'chat',
                    component: ChatView
                },
                {
                    path: '/people',
                    name: 'people',
                    component: PeopleView
                },
                {
                    path: '/account',
                    name: 'account',
                    component: AccountView
                },
                {
                    path: '/settings',
                    name: 'settings',
                    component: SettingsView
                }
            ]
        },
    ]
});

export default router;