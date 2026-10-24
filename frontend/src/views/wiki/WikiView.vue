<template>
  <div class="actions wrapper-center">
    <RouterLink to="/wiki/create-article" v-if="userStore.permissions.includes('create_wiki_article')">
      <i class="pi pi-plus" v-tooltip.top="t('wiki.create_article')" />
    </RouterLink>
    <RouterLink to="/wiki/search">
      <i class="pi pi-search" v-tooltip.top="t('wiki.search')" />
    </RouterLink>
  </div>
  <div v-if="wikiArticle" class="wiki-article">
    <ContentContainer :containerTitle="wikiArticle.title" class="main-content">
      <template v-if="userStore.permissions.includes('edit_wiki_article')" #top-right>
        <RouterLink :to="`/wiki/article/${wikiArticle.id}/edit`" v-tooltip.top="t('wiki.edit_article')">
          <i class="pi pi-pen-to-square" style="color: white" />
        </RouterLink>
      </template>
      <BBCodeRenderer :content="wikiArticle.body" />
    </ContentContainer>
    <ContentContainer :containerTitle="t('wiki.article_metadata')" class="sidebar">
      <dl class="metadata-list">
        <dt>{{ t('general.created_at') }}</dt>
        <dd>{{ wikiArticle.created_at ? formatDate(wikiArticle.created_at) : t('general.not_available') }}</dd>
        <dt>{{ t('wiki.created_by') }}</dt>
        <dd><UsernameEnriched :user="wikiArticle.created_by" /></dd>
        <dt>{{ t('general.updated_at') }}</dt>
        <dd>{{ wikiArticle.updated_at ? formatDate(wikiArticle.updated_at) : t('general.not_available') }}</dd>
        <dt>{{ t('wiki.updated_by') }}</dt>
        <dd><UsernameEnriched :user="wikiArticle.updated_by" /></dd>
      </dl>
    </ContentContainer>
    <ContentContainer
      v-if="wikiArticle.similar_wiki_articles.length > 0 || canLinkSimilarArticles"
      :containerTitle="t('wiki.similar_articles')"
      class="sidebar"
    >
      <template v-if="canLinkSimilarArticles" #top-right>
        <i class="pi pi-plus cursor-pointer" style="color: white" v-tooltip.top="t('wiki.link_similar_article')" @click="linkDialogVisible = true" />
      </template>
      <ul v-if="wikiArticle.similar_wiki_articles.length > 0" class="similar-list">
        <li v-for="similar in wikiArticle.similar_wiki_articles" :key="similar.id">
          <RouterLink :to="`/wiki/article/${similar.id}`">{{ similar.title }}</RouterLink>
          <i
            v-if="canLinkSimilarArticles"
            class="pi pi-times-circle cursor-pointer"
            style="font-size: 0.8rem; margin-left: 5px"
            v-tooltip.top="t('wiki.unlink_similar_article')"
            @click="unlinkArticle(similar.id)"
          />
        </li>
      </ul>
    </ContentContainer>
  </div>

  <LinkSimilarWikiArticleDialog
    v-if="wikiArticle"
    v-model:visible="linkDialogVisible"
    :articleId="wikiArticle.id"
    :linkedArticleIds="wikiArticle.similar_wiki_articles.map((s) => s.id)"
    @linked="onArticleLinked"
  />
</template>

<script setup lang="ts">
import BBCodeRenderer from '@/components/community/BBCodeRenderer.vue'
import ContentContainer from '@/components/ContentContainer.vue'
import UsernameEnriched from '@/components/user/UsernameEnriched.vue'
import LinkSimilarWikiArticleDialog from '@/components/wiki/LinkSimilarWikiArticleDialog.vue'
import { computed, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { useI18n } from 'vue-i18n'
import { getWikiArticle, unlinkSimilarWikiArticles, type UserLite, type WikiArticle, type WikiSearchResult } from '@/services/api-schema'
import { formatDate } from '@/services/helpers'
import { showToast } from '@/main'

interface SimilarWikiArticle {
  id: number
  title: string
}
type WikiArticleWithSimilar = WikiArticle & {
  similar_wiki_articles: SimilarWikiArticle[]
}
type WikiArticleWithMetadata = WikiArticleWithSimilar & {
  created_by?: UserLite | null
  updated_by?: UserLite | null
}

const { t } = useI18n()
const route = useRoute()
const userStore = useUserStore()

const wikiArticle = ref<WikiArticleWithMetadata>()
const linkDialogVisible = ref(false)

const canLinkSimilarArticles = computed(() => userStore.permissions.includes('link_similar_wiki_articles'))

const fetchWikiArticle = () => {
  getWikiArticle(parseInt(route.params.id as string)).then((article) => {
    wikiArticle.value = article as WikiArticleWithMetadata
  })
}

const onArticleLinked = (linked: WikiSearchResult) => {
  if (!wikiArticle.value) return
  wikiArticle.value.similar_wiki_articles.push({ id: linked.id, title: linked.title })
  wikiArticle.value.similar_wiki_articles.sort((a, b) => a.title.localeCompare(b.title))
}

const unlinkArticle = (similarId: number) => {
  if (!wikiArticle.value) return
  const currentArticle = wikiArticle.value
  unlinkSimilarWikiArticles({
    wiki_article_id_1: currentArticle.id,
    wiki_article_id_2: similarId,
  }).then(() => {
    currentArticle.similar_wiki_articles = currentArticle.similar_wiki_articles.filter((s) => s.id !== similarId)
    showToast('Success', t('wiki.similar_article_unlinked'), 'success', 4000)
  })
}

watch(() => route.params.id, fetchWikiArticle, { immediate: true })
</script>

<style scoped>
.wiki-article {
  margin: 20px 0;
  display: flex;
  gap: 15px;
  align-items: flex-start;
}
.main-content {
  flex: 1;
  min-width: 0;
}
.sidebar {
  width: 250px;
  flex-shrink: 0;
}
.similar-list {
  margin: 0;
  padding-left: 20px;
}
.metadata-list {
  margin: 0;
}
.metadata-list dt {
  font-weight: bold;
  margin-top: 8px;
}
.metadata-list dt:first-child {
  margin-top: 0;
}
.metadata-list dd {
  margin: 2px 0 0;
  overflow-wrap: anywhere;
}
.similar-list li {
  margin: 4px 0;
}
.actions {
  i {
    margin: 0 5px;
    color: white;
  }
}
@media (max-width: 768px) {
  .wiki-article {
    flex-direction: column;
  }
  .sidebar {
    width: 100%;
  }
}
</style>
