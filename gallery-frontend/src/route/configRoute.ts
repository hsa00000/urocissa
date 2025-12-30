// src/route/configRoute.ts

import { RouteRecordRaw } from 'vue-router'
import 'vue-router'

import ConfigPage from '@/components/Page/ConfigPage.vue'

export const configRoute: RouteRecordRaw = {
  path: '/config',
  component: ConfigPage,
  name: 'config',

  meta: {
    basicString: null,
    level: 1,
    baseName: 'config',
    getParentPage: (route) => {
      return {
        name: 'home',
        params: { hash: undefined, subhash: undefined },
        query: route.query
      }
    },
    getChildPage: (route) => {
      return {
        name: 'config',
        params: { hash: undefined, subhash: undefined },
        query: route.query
      }
    }
  }
}
