/**
 * Composable for algorithm-related logic
 * Provides algorithm configuration and display utilities
 */

import { computed } from 'vue'
import type { AlgorithmId } from '@/types/superLotto'

export interface AlgorithmInfo {
  id: string
  name: string
  description: string
  category: 'statistical' | 'machine_learning' | 'hybrid'
  difficulty: 'low' | 'medium' | 'high'
}

const ALGORITHM_CONFIGS: Record<string, AlgorithmInfo> = {
  WEIGHTED_FREQUENCY: {
    id: 'WEIGHTED_FREQUENCY',
    name: '加权频率分析',
    description: '基于历史频率分析，考虑时间衰减因子',
    category: 'statistical',
    difficulty: 'low'
  },
  PATTERN_BASED: {
    id: 'PATTERN_BASED',
    name: '模式分析',
    description: '基于数字模式和奇偶比例识别',
    category: 'statistical',
    difficulty: 'medium'
  },
  MARKOV_CHAIN: {
    id: 'MARKOV_CHAIN',
    name: '马尔可夫链',
    description: '基于状态转移概率分析',
    category: 'statistical',
    difficulty: 'high'
  },
  ENSEMBLE: {
    id: 'ENSEMBLE',
    name: '集成方法',
    description: '多算法综合结果，投票机制',
    category: 'hybrid',
    difficulty: 'high'
  },
  HOT_NUMBERS: {
    id: 'HOT_NUMBERS',
    name: '热号预测',
    description: '基于热门号码分析',
    category: 'statistical',
    difficulty: 'low'
  },
  COLD_NUMBERS: {
    id: 'COLD_NUMBERS',
    name: '冷号预测',
    description: '基于冷门号码分析',
    category: 'statistical',
    difficulty: 'low'
  },
  POSITION_ANALYSIS: {
    id: 'POSITION_ANALYSIS',
    name: '位置分析',
    description: '基于位置分布和位置频率',
    category: 'statistical',
    difficulty: 'medium'
  }
}

export function useAlgorithm() {
  const getAlgorithmInfo = (algorithmId: string): AlgorithmInfo | null => {
    return ALGORITHM_CONFIGS[algorithmId] || null
  }

  const getAlgorithmName = (algorithmId: string): string => {
    return ALGORITHM_CONFIGS[algorithmId]?.name || algorithmId
  }

  const getAlgorithmDescription = (algorithmId: string): string => {
    return ALGORITHM_CONFIGS[algorithmId]?.description || ''
  }

  const getAllAlgorithms = computed(() => Object.values(ALGORITHM_CONFIGS))

  const getAlgorithmsByCategory = (category: AlgorithmInfo['category']) => {
    return Object.values(ALGORITHM_CONFIGS).filter(a => a.category === category)
  }

  const getAlgorithmsByDifficulty = (difficulty: AlgorithmInfo['difficulty']) => {
    return Object.values(ALGORITHM_CONFIGS).filter(a => a.difficulty === difficulty)
  }

  const getDefaultAlgorithm = (): string => {
    return 'WEIGHTED_FREQUENCY'
  }

  const getRecommendedAlgorithms = (): string[] => {
    return ['WEIGHTED_FREQUENCY', 'ENSEMBLE', 'HOT_NUMBERS']
  }

  return {
    getAlgorithmInfo,
    getAlgorithmName,
    getAlgorithmDescription,
    getAllAlgorithms,
    getAlgorithmsByCategory,
    getAlgorithmsByDifficulty,
    getDefaultAlgorithm,
    getRecommendedAlgorithms
  }
}
