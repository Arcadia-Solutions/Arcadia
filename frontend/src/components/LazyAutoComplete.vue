<template>
  <!--
    Drop-in replacement for PrimeVue's AutoComplete that keeps the (fairly large)
    primevue/autocomplete + virtualscroller modules out of the initial bundle. Until the field is
    first focused we render a plain input carrying the exact same classes, so it is
    pixel-identical to the real AutoComplete and swapping it in causes no layout shift. The
    placeholder stays mounted while the real component's chunk downloads.
  -->
  <component
    :is="AutoCompleteComponent"
    v-if="AutoCompleteComponent"
    ref="realRef"
    v-bind="$attrs"
    :modelValue="modelValue"
    :placeholder="placeholder"
    @update:modelValue="emit('update:modelValue', $event)"
  >
    <template v-for="(_, name) in $slots" #[name]="slotProps">
      <slot :name="name" v-bind="slotProps ?? {}" />
    </template>
  </component>
  <div v-else class="p-autocomplete p-component p-inputwrapper lazy-autocomplete" @click="activate">
    <input
      ref="placeholderRef"
      type="text"
      class="p-inputtext p-component"
      :class="{ 'p-inputtext-sm p-inputfield-sm': $attrs.size === 'small' }"
      :placeholder="placeholder"
      :value="modelValue"
      v-bind="placeholderAttrs"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @focus="activate"
    />
  </div>
</template>

<script setup lang="ts">
import { shallowRef, ref, watch, markRaw, computed, useAttrs, type ComponentPublicInstance } from 'vue'

defineOptions({ inheritAttrs: false })

const props = defineProps<{
  modelValue?: string
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const attrs = useAttrs()
// AutoComplete props (suggestions, optionLabel, size...) would end up as bogus DOM attributes on the
// plain input, so it only receives the listeners (e.g. @input, @keydown.enter) and class/style.
const placeholderAttrs = computed(() => Object.fromEntries(Object.entries(attrs).filter(([key]) => /^on[A-Z]/.test(key) || key === 'class' || key === 'style')))

// Loads (and parses) primevue/autocomplete + virtualscroller only when the field is first used.
const AutoCompleteComponent = shallowRef()
const realRef = ref<ComponentPublicInstance | null>(null)
const placeholderRef = ref<HTMLInputElement | null>(null)
// Only steal focus back if the user is still on this field when the chunk arrives.
let restoreFocus = false

let loading = false
const activate = async () => {
  if (AutoCompleteComponent.value || loading) return
  loading = true
  try {
    const mod = await import('primevue/autocomplete')
    restoreFocus = document.activeElement === placeholderRef.value
    AutoCompleteComponent.value = markRaw(mod.default ?? mod)
  } catch {
    loading = false
  }
}

// Move focus into the real AutoComplete once it has mounted, and replay the text typed while the
// chunk was downloading (the placeholder never emitted `complete` for it).
watch(realRef, (instance) => {
  if (!instance || !restoreFocus) return
  ;(instance.$el as HTMLElement).querySelector('input')?.focus()
  const autoComplete = instance as unknown as { search: (event: Event, query: string, source: string) => void }
  if (props.modelValue) autoComplete.search(new Event('input'), props.modelValue, 'input')
})
</script>

<style scoped>
/* Match the real .p-autocomplete box even if its stylesheet has not loaded yet. */
.lazy-autocomplete {
  display: inline-flex;
}
</style>
