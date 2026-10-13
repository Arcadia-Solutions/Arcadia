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
    <template #option="slotProps">
      <slot name="option" v-bind="slotProps" />
    </template>
  </component>
  <div v-else class="p-autocomplete p-component p-inputwrapper lazy-autocomplete" @click="activate">
    <input
      type="text"
      class="p-inputtext p-component p-inputtext-sm p-inputfield-sm"
      :placeholder="placeholder"
      :value="modelValue"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @focus="activate"
    />
  </div>
</template>

<script setup lang="ts">
import { shallowRef, ref, watch, markRaw, type ComponentPublicInstance } from 'vue'

defineOptions({ inheritAttrs: false })

defineProps<{
  modelValue?: string
  placeholder?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

// Loads (and parses) primevue/autocomplete + virtualscroller only when the field is first used.
const AutoCompleteComponent = shallowRef()
const realRef = ref<ComponentPublicInstance | null>(null)

let loading = false
const activate = async () => {
  if (AutoCompleteComponent.value || loading) return
  loading = true
  const mod = await import('primevue/autocomplete')
  AutoCompleteComponent.value = markRaw(mod.default ?? mod)
}

// Move focus into the real AutoComplete once it has mounted.
watch(realRef, (instance) => {
  const el = instance?.$el as HTMLElement | undefined
  el?.querySelector('input')?.focus()
})
</script>

<style scoped>
/* Match the real .p-autocomplete box even if its stylesheet has not loaded yet. */
.lazy-autocomplete {
  display: inline-flex;
}
</style>
