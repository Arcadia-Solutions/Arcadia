import { createRouter, createWebHistory, type RouteRecordInfo } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import { endPageNavigation, startPageProgress } from '@/services/progressBar'

export interface RouteNamedMap {
  TitleGroup: RouteRecordInfo<'TitleGroup', '/title-group/:id', { id: string | number }>
  Series: RouteRecordInfo<'Series', '/series/:id', { id: string | number }, { id: string }>
  Artist: RouteRecordInfo<'Artist', '/artist/:id', { id: string | number }, { id: string }>
  UploadTorrent: RouteRecordInfo<'UploadTorrent', '/upload'>
  NewTorrentRequest: RouteRecordInfo<'NewTorrentRequest', '/new-torrent-request'>
  EditCssSheet: RouteRecordInfo<'EditCssSheet', '/css-sheets/:name/edit', { name: string }>
  Shop: RouteRecordInfo<'Shop', '/shop'>
}

declare module 'vue-router' {
  interface TypesConfig {
    RouteNamedMap: RouteNamedMap
  }
}

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'Login',
      component: () => import('../views/AuthView.vue'),
    },
    {
      path: '/register',
      name: 'Register',
      component: () => import('../views/AuthView.vue'),
    },
    {
      path: '/apply',
      name: 'Apply',
      component: () => import('../views/AuthView.vue'),
    },
    {
      path: '/reset-password',
      name: 'Reset password',
      component: () => import('../views/AuthView.vue'),
    },
    {
      path: '/oauth/authorize',
      name: 'OAuthAuthorize',
      component: () => import('../views/OAuthAuthorizeView.vue'),
    },
    {
      path: '/',
      name: 'Home',
      component: HomeView,
    },
    {
      path: '/title-group/:id',
      name: 'TitleGroup',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/TitleGroupView.vue'),
    },
    {
      path: '/title-group-comments',
      name: 'UserTitleGroupComments',
      meta: {
        documentTitle: 'Title group comments',
      },
      component: () => import('../views/UserTitleGroupCommentsView.vue'),
    },
    {
      path: '/title-group-tags',
      name: 'TitleGroupTags',
      meta: {
        documentTitle: 'Title group tags',
      },
      component: () => import('../views/TitleGroupTagsView.vue'),
    },
    {
      path: '/torrents',
      name: 'Torrents',
      component: () => import('../views/TorrentSearchView.vue'),
    },
    {
      path: '/torrent/:id',
      name: 'Torrent',
      component: () => import('../views/TorrentIdRedirectToTitleGroupView.vue'),
    },
    {
      path: '/torrent-requests',
      name: 'TorrentRequests',
      meta: {
        documentTitle: 'Torrent requests',
      },
      component: () => import('../views/torrent_request/TorrentRequestSearchView.vue'),
    },
    {
      path: '/new-torrent-request',
      name: 'NewTorrentRequest',
      meta: {
        documentTitle: 'New torrent request',
      },
      component: () => import('../views/torrent_request/NewTorrentRequestView.vue'),
    },
    {
      path: '/torrent-request/:id',
      name: 'TorrentRequest',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/torrent_request/TorrentRequestView.vue'),
    },
    {
      path: '/request-comments',
      name: 'UserTorrentRequestComments',
      meta: {
        documentTitle: 'Request comments',
      },
      component: () => import('../views/torrent_request/UserTorrentRequestCommentsView.vue'),
    },
    {
      path: '/new-series',
      name: 'CreateSeries',
      component: () => import('../views/series/CreateOrEditSeriesView.vue'),
    },
    {
      path: '/series/:id',
      name: 'Series',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/series/SeriesView.vue'),
    },
    {
      path: '/series',
      name: 'Search series',
      component: () => import('../views/series/SeriesSearchView.vue'),
    },
    {
      path: '/artists',
      name: 'Search artists',
      component: () => import('../views/artists/ArtistsSearchView.vue'),
    },
    {
      path: '/artist/:id',
      name: 'Artist',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/ArtistView.vue'),
    },
    {
      path: '/upload',
      name: 'UploadTorrent',
      meta: {
        documentTitle: 'Upload torrent',
      },
      component: () => import('../views/UploadTorrentView.vue'),
    },
    {
      path: '/users',
      name: 'Users',
      meta: {
        documentTitle: 'Users',
      },
      component: () => import('../views/UserSearchView.vue'),
    },
    {
      path: '/user/:id',
      name: 'User',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/UserView.vue'),
    },
    {
      path: '/forum',
      name: 'Forum',
      component: () => import('../views/forum/ForumOverviewView.vue'),
    },
    {
      path: '/forum/search',
      name: 'ForumSearch',
      component: () => import('../views/forum/ForumSearch.vue'),
    },
    {
      path: '/forum/posts',
      name: 'UserForumPosts',
      meta: {
        documentTitle: 'Forum posts',
      },
      component: () => import('../views/forum/UserForumPostsView.vue'),
    },
    {
      path: '/forum/sub-category/:id',
      name: 'ForumSubCategory',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/forum/ForumSubCategoryView.vue'),
    },
    {
      path: '/forum/thread/:id',
      name: 'ForumThread',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/forum/ForumThreadView.vue'),
    },
    {
      path: '/forum/thread/new',
      name: 'NewForumThread',
      meta: {
        documentTitle: 'New forum thread',
      },
      component: () => import('../views/forum/NewForumThreadView.vue'),
    },
    {
      path: '/forum/category/new',
      name: 'NewForumCategory',
      meta: {
        documentTitle: 'Create forum category',
      },
      component: () => import('../views/forum/CreateOrEditForumCategoryView.vue'),
    },
    {
      path: '/forum/category/:id/edit',
      name: 'EditForumCategory',
      meta: {
        documentTitle: 'Edit forum category',
      },
      component: () => import('../views/forum/CreateOrEditForumCategoryView.vue'),
    },
    {
      path: '/forum/sub-category/new',
      name: 'NewForumSubCategory',
      meta: {
        documentTitle: 'Create forum subcategory',
      },
      component: () => import('../views/forum/CreateOrEditForumSubCategoryView.vue'),
    },
    {
      path: '/forum/sub-category/:id/edit',
      name: 'EditForumSubCategory',
      meta: {
        documentTitle: 'Edit forum subcategory',
      },
      component: () => import('../views/forum/CreateOrEditForumSubCategoryView.vue'),
    },
    {
      path: '/wiki/search',
      name: 'WikiSearch',
      meta: {
        documentTitle: 'Wiki Search',
      },
      component: () => import('../views/wiki/WikiSearchView.vue'),
    },
    {
      path: '/wiki/article/:id',
      name: 'WikiArticle',
      meta: {
        documentTitle: 'Wiki',
      },
      component: () => import('../views/wiki/WikiView.vue'),
    },
    {
      path: '/wiki/article/:id/edit',
      name: 'EditWikiArticle',
      meta: {
        documentTitle: 'Edit Wiki Article',
      },
      component: () => import('../views/wiki/CreateOrEditWikiArticle.vue'),
    },
    {
      path: '/wiki/create-article',
      name: 'CreateWikiArticle',
      meta: {
        documentTitle: 'Create Wiki Article',
      },
      component: () => import('../views/wiki/CreateOrEditWikiArticle.vue'),
    },
    {
      path: '/conversation/new',
      name: 'NewConversation',
      meta: {
        documentTitle: 'New conversation',
      },
      component: () => import('../views/conversation/NewConversationView.vue'),
    },
    {
      path: '/conversation/:id',
      name: 'Conversation',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/conversation/ConversationView.vue'),
    },
    {
      path: '/staff-pm/:id',
      name: 'Staff PM',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/staff_pm/StaffPmView.vue'),
    },
    {
      path: '/conversations',
      name: 'Conversations',
      component: () => import('../views/conversation/ConversationsView.vue'),
    },
    {
      path: '/collage/:id',
      name: 'Collage',
      meta: {
        dynamicDocumentTitle: true,
      },
      component: () => import('../views/collage/CollageView.vue'),
    },
    {
      path: '/collages',
      name: 'Collages',
      component: () => import('../views/collage/CollageSearchView.vue'),
    },
    {
      path: '/new-collage',
      name: 'NewCollage',
      meta: {
        documentTitle: 'New collage',
      },
      component: () => import('../views/collage/NewCollageView.vue'),
    },
    {
      path: '/notifications',
      name: 'Notifications',
      meta: {
        documentTitle: 'Notifications',
      },
      component: () => import('../views/NotificationsView.vue'),
    },
    {
      path: '/subscriptions',
      name: 'Subscriptions',
      meta: {
        documentTitle: 'Subscriptions',
      },
      component: () => import('../views/SubscriptionsView.vue'),
    },
    {
      path: '/user-settings',
      name: 'UserSettings',
      meta: {
        documentTitle: 'User settings',
      },
      component: () => import('../views/UserSettingsView.vue'),
    },
    {
      path: '/stats',
      name: 'Stats',
      meta: {
        documentTitle: 'Stats',
      },
      component: () => import('../views/StatsView.vue'),
    },
    {
      path: '/staff-dashboard',
      name: 'StaffDashboard',
      component: () => import('../views/StaffDashboardView.vue'),
    },
    {
      path: '/staff-pms',
      name: 'StaffPms',
      component: () => import('../views/staff_pm/StaffPmsView.vue'),
    },
    {
      path: '/staff-pms/new',
      name: 'NewStaffPm',
      component: () => import('../views/staff_pm/NewStaffPMView.vue'),
    },
    {
      path: '/css-sheets/new',
      name: 'CreateCssSheet',
      meta: {
        documentTitle: 'Create CSS sheet',
      },
      component: () => import('../views/CreateOrEditCssSheetView.vue'),
    },
    {
      path: '/css-sheets/:name/edit',
      name: 'EditCssSheet',
      meta: {
        documentTitle: 'Edit CSS sheet',
      },
      component: () => import('../views/CreateOrEditCssSheetView.vue'),
    },
    {
      path: '/invitations',
      name: 'Invitations',
      meta: {
        documentTitle: 'Invitations',
      },
      component: () => import('../views/InvitationsView.vue'),
    },
    {
      path: '/shop',
      name: 'Shop',
      meta: {
        documentTitle: 'Bonus Points Shop',
      },
      component: () => import('../views/ShopView.vue'),
    },
    {
      path: '/:pathMatch(.*)*',
      name: 'NotFound',
      meta: {
        documentTitle: 'Page not found',
      },
      component: () => import('../views/NotFoundView.vue'),
    },
  ],
})

router.beforeEach((to, from) => {
  if (to.fullPath !== from.fullPath) {
    startPageProgress()
  }
})

router.afterEach(() => endPageNavigation())

router.onError(() => endPageNavigation())

export default router
