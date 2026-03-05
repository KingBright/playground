import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { Button } from './Button'

describe('Button', () => {
  it('renders correctly', () => {
    render(<Button>Click me</Button>)
    expect(screen.getByText('Click me')).toBeInTheDocument()
  })

  it('handles click events', () => {
    const handleClick = vi.fn()
    render(<Button onClick={handleClick}>Click me</Button>)
    screen.getByText('Click me').click()
    expect(handleClick).toHaveBeenCalledTimes(1)
  })

  it('is disabled when loading', () => {
    render(<Button loading>Loading</Button>)
    expect(screen.getByText('Loading')).toBeDisabled()
  })

  it('renders icon when provided', () => {
    render(<Button icon="add">Add</Button>)
    expect(screen.getByText('add')).toBeInTheDocument()
  })

  it('applies correct variant styles', () => {
    const { container } = render(<Button variant="danger">Danger</Button>)
    expect(container.firstChild).toHaveClass('bg-red-600')
  })

  it('applies full width class when fullWidth is true', () => {
    const { container } = render(<Button fullWidth>Full Width</Button>)
    expect(container.firstChild).toHaveClass('w-full')
  })
})
