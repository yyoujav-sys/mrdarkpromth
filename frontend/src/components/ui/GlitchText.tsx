import React from 'react'

interface GlitchTextProps {
    text: string
    as?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6' | 'p' | 'span' | 'div'
    className?: string
}

export const GlitchText: React.FC<GlitchTextProps> = ({ text, as: Component = 'span', className = '' }) => {
    return (
        <Component className={`relative inline-block group ${className}`}>
            <span className="relative z-10">{text}</span>
            <span className="absolute top-0 left-0 -z-10 w-full h-full text-neon-green opacity-0 group-hover:opacity-100 group-hover:animate-glitch translate-x-[2px]">
                {text}
            </span>
            <span className="absolute top-0 left-0 -z-10 w-full h-full text-neon-red opacity-0 group-hover:opacity-100 group-hover:animate-glitch translate-x-[-2px] animation-delay-75">
                {text}
            </span>
        </Component>
    )
}
