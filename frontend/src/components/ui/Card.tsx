import React from 'react'
import { cn } from '@/lib/utils'

interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'glass' | 'neon'
}

export const Card: React.FC<CardProps> = ({ 
  className, 
  variant = 'default',
  children, 
  ...props 
}) => {
  const variants = {
    default: 'bg-gray-800 border border-gray-700',
    glass: 'bg-gray-800/50 backdrop-blur-sm border border-gray-700',
    neon: 'bg-gray-800 border border-purple-600 shadow-lg shadow-purple-500/20'
  }
  
  return (
    <div
      className={cn(
        'rounded-xl shadow-lg',
        variants[variant],
        className
      )}
      {...props}
    >
      {children}
    </div>
  )
}

export const CardHeader: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({ 
  className, 
  children, 
  ...props 
}) => {
  return (
    <div
      className={cn('p-6 pb-4', className)}
      {...props}
    >
      {children}
    </div>
  )
}

export const CardContent: React.FC<React.HTMLAttributes<HTMLDivElement>> = ({ 
  className, 
  children, 
  ...props 
}) => {
  return (
    <div
      className={cn('p-6 pt-0', className)}
      {...props}
    >
      {children}
    </div>
  )
}

export const CardTitle: React.FC<React.HTMLAttributes<HTMLHeadingElement>> = ({ 
  className, 
  children, 
  ...props 
}) => {
  return (
    <h3
      className={cn('text-xl font-semibold text-gray-100', className)}
      {...props}
    >
      {children}
    </h3>
  )
}
