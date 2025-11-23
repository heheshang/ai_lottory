import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import AlgorithmSelector from '@/components/super-lotto/AlgorithmSelector.vue'

describe('AlgorithmSelector Component', () => {
  let wrapper: any

  beforeEach(() => {
    wrapper = mount(AlgorithmSelector, {
      props: {
        modelValue: ['WEIGHTED_FREQUENCY'],
        algorithms: [
          { id: 'WEIGHTED_FREQUENCY', name: '加权频率', description: '基于历史频率分析' },
          { id: 'PATTERN_BASED', name: '模式分析', description: '基于号码模式识别' },
          { id: 'HOT_NUMBERS', name: '热号分析', description: '基于热号统计分析' }
        ]
      }
    })
  })

  it('should render correctly', () => {
    expect(wrapper.find('[data-testid="algorithm-selector"]').exists()).toBe(true)
    expect(wrapper.findAll('[data-testid="algorithm-item"]')).toHaveLength(3)
  })

  it('should display algorithm information correctly', () => {
    const algorithmItems = wrapper.findAll('[data-testid="algorithm-item"]')

    expect(algorithmItems[0].text()).toContain('加权频率')
    expect(algorithmItems[0].text()).toContain('基于历史频率分析')

    expect(algorithmItems[1].text()).toContain('模式分析')
    expect(algorithmItems[2].text()).toContain('热号分析')
  })

  it('should emit update event when selection changes', async () => {
    const checkboxes = wrapper.findAll('input[type="checkbox"]')

    // Click second algorithm
    await checkboxes[1].setValue(true)
    await nextTick()

    expect(wrapper.emitted('update:modelValue')).toBeTruthy()
    expect(wrapper.emitted('update:modelValue')[0]).toEqual([
      ['WEIGHTED_FREQUENCY', 'PATTERN_BASED']
    ])
  })

  it('should handle multiple selections', async () => {
    const checkboxes = wrapper.findAll('input[type="checkbox"]')

    // Select multiple algorithms
    await checkboxes[1].setValue(true)
    await checkboxes[2].setValue(true)
    await nextTick()

    const selectedAlgorithms = wrapper.emitted('update:modelValue')[1]
    expect(selectedAlgorithms[0]).toContain('WEIGHTED_FREQUENCY')
    expect(selectedAlgorithms[0]).toContain('PATTERN_BASED')
    expect(selectedAlgorithms[0]).toContain('HOT_NUMBERS')
  })

  it('should display selection count badge', async () => {
    const checkboxes = wrapper.findAll('input[type="checkbox"]')

    // Initially should show 1 selected
    expect(wrapper.find('[data-testid="selection-count"]').text()).toContain('已选: 1')

    // Add another selection
    await checkboxes[1].setValue(true)
    await nextTick()

    expect(wrapper.find('[data-testid="selection-count"]').text()).toContain('已选: 2')
  })

  it('should show algorithm details on hover', async () => {
    const firstItem = wrapper.find('[data-testid="algorithm-item"]')

    // Trigger hover
    await firstItem.trigger('mouseenter')
    await nextTick()

    expect(wrapper.find('[data-testid="algorithm-tooltip"]').exists()).toBe(true)
    expect(wrapper.find('[data-testid="algorithm-tooltip"]').text()).toContain('基于历史频率分析')
  })

  it('should validate minimum selection', async () => {
    const checkboxes = wrapper.findAll('input[type="checkbox"]')

    // Try to deselect all algorithms
    await checkboxes[0].setValue(false)
    await nextTick()

    // Should maintain at least one selection
    const selectedCount = wrapper.emitted('update:modelValue')?.[0]?.[0]?.length || 0
    expect(selectedCount).toBeGreaterThanOrEqual(1)
  })

  it('should handle keyboard navigation', async () => {
    const firstItem = wrapper.find('[data-testid="algorithm-item"]')

    // Focus first item
    await firstItem.trigger('focus')
    await firstItem.trigger('keydown.space')
    await nextTick()

    expect(wrapper.emitted('update:modelValue')).toBeTruthy()
  })

  it('should be accessible', () => {
    const checkboxes = wrapper.findAll('input[type="checkbox"]')

    checkboxes.forEach((checkbox: any, index: number) => {
      expect(checkbox.attributes('aria-label')).toBeTruthy()
      expect(checkbox.attributes('aria-checked')).toBeTruthy()
    })

    expect(wrapper.find('[data-testid="algorithm-selector"]').attributes('role')).toBe('group')
    expect(wrapper.find('[data-testid="algorithm-selector"]').attributes('aria-label')).toBe('选择预测算法')
  })
})