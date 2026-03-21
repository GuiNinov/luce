interface BoardBackgroundProps {
  width: number
  height: number
}

export function BoardBackground({ width, height }: BoardBackgroundProps) {
  return (
    <div className="absolute inset-0 pointer-events-none">
      {/* Completed Tasks Zone - Left */}
      <div 
        className="absolute top-0 bottom-0 bg-gradient-to-r from-gray-50 to-gray-100/50"
        style={{ 
          left: 0, 
          width: '200px',
          transform: 'translateX(-50px)'
        }}
      >
        <div className="absolute top-4 left-4 text-xs font-medium text-gray-400 uppercase tracking-wide">
          Completed
        </div>
      </div>
      
      {/* Active Work Zone - Center */}
      <div 
        className="absolute top-0 bottom-0 bg-gradient-to-r from-blue-50/30 via-blue-50/60 to-blue-50/30"
        style={{ 
          left: '250px', 
          width: '800px'
        }}
      >
        <div className="absolute top-4 left-4 text-xs font-medium text-blue-600 uppercase tracking-wide">
          Active Work
        </div>
        
        {/* Spotlight effect for emphasis */}
        <div 
          className="absolute top-20 left-1/2 transform -translate-x-1/2 w-96 h-96 bg-blue-100/20 rounded-full blur-3xl opacity-50"
        />
        
        {/* Subtle grid pattern for active area */}
        <div 
          className="absolute inset-0 opacity-20"
          style={{
            backgroundImage: `
              linear-gradient(rgba(59, 130, 246, 0.1) 1px, transparent 1px),
              linear-gradient(90deg, rgba(59, 130, 246, 0.1) 1px, transparent 1px)
            `,
            backgroundSize: '40px 40px'
          }}
        />
        
        {/* Animated flow lines */}
        <div className="absolute inset-0 overflow-hidden">
          <div className="absolute top-1/2 left-0 w-full h-px bg-gradient-to-r from-transparent via-blue-300/30 to-transparent animate-pulse" />
          <div className="absolute top-1/3 left-0 w-full h-px bg-gradient-to-r from-transparent via-blue-200/20 to-transparent animate-pulse delay-1000" />
          <div className="absolute top-2/3 left-0 w-full h-px bg-gradient-to-r from-transparent via-blue-200/20 to-transparent animate-pulse delay-2000" />
        </div>
      </div>
      
      {/* Future Work Zone - Right */}
      <div 
        className="absolute top-0 bottom-0 bg-gradient-to-r from-green-50/20 to-green-50/40"
        style={{ 
          left: '1050px', 
          width: '400px'
        }}
      >
        <div className="absolute top-4 left-4 text-xs font-medium text-green-600 uppercase tracking-wide">
          Future Work
        </div>
      </div>
      
      {/* Zone Dividers */}
      <div 
        className="absolute top-0 bottom-0 w-px bg-gradient-to-b from-transparent via-gray-300/50 to-transparent"
        style={{ left: '250px' }}
      />
      <div 
        className="absolute top-0 bottom-0 w-px bg-gradient-to-b from-transparent via-blue-300/50 to-transparent"
        style={{ left: '1050px' }}
      />
    </div>
  )
}