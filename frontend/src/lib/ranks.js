import unrankedIcon from '../assets/ranks/unranked.webp'
import bronzeIcon from '../assets/ranks/bronze.webp'
import silverIcon from '../assets/ranks/silver.webp'
import goldIcon from '../assets/ranks/gold.webp'
import platinumIcon from '../assets/ranks/platinum.webp'
import diamondIcon from '../assets/ranks/diamond.webp'
import masterIcon from '../assets/ranks/master.webp'
import grandmasterIcon from '../assets/ranks/grandmaster.webp'
import championIcon from '../assets/ranks/champion.webp'

export const overwatchRanks = [
  'Unranked',
  'Bronze',
  'Silver',
  'Gold',
  'Platinum',
  'Diamond',
  'Master',
  'Grandmaster',
  'Champion'
]

const rankIcons = {
  Unranked: unrankedIcon,
  Bronze: bronzeIcon,
  Silver: silverIcon,
  Gold: goldIcon,
  Platinum: platinumIcon,
  Diamond: diamondIcon,
  Master: masterIcon,
  Grandmaster: grandmasterIcon,
  Champion: championIcon
}

const rankElo = {
  Unranked: null,
  Bronze: 1000,
  Silver: 1500,
  Gold: 2000,
  Platinum: 2500,
  Diamond: 3000,
  Master: 3500,
  Grandmaster: 4000,
  Champion: 4500,
}

export function getRankIcon(rank) {
  return rankIcons[rank] || unrankedIcon
}

export function getRankElo(rank) {
  return rankElo[rank] ?? null
}
