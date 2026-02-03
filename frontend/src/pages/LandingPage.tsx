import React from 'react'
import { Link } from 'react-router-dom'
import { 
  Bot, 
  Zap, 
  Shield, 
  Lock, 
  Github,
  ArrowRight,
  Sparkles,
  Terminal,
  Cpu,
  Network
} from 'lucide-react'

export const LandingPage: React.FC = () => {
  return (
    <div className="min-h-screen" style={{background: 'var(--bg-primary)'}}>
      {/* Navigation */}
      <nav className="navbar">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex items-center justify-between h-16">
            <div className="flex items-center space-x-3">
              <div className="p-2 rounded-full pulse" style={{background: 'rgba(255, 0, 110, 0.2)'}}>
                <Bot className="h-6 w-6" style={{color: 'var(--accent-primary)'}} />
              </div>
              <span className="navbar-brand">MR.DarkPromth</span>
            </div>
            <div className="navbar-nav">
              <Link to="/login" className="nav-link">Sign In</Link>
              <Link to="/register" className="btn btn-primary">Get Started</Link>
            </div>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <section className="relative overflow-hidden py-20">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <div className="relative z-10">
            <div className="inline-flex items-center space-x-2 px-4 py-2 rounded-full mb-8" 
                 style={{background: 'rgba(255, 0, 110, 0.1)', border: '1px solid var(--accent-primary)'}}>
              <Sparkles className="h-4 w-4" style={{color: 'var(--accent-primary)'}} />
              <span style={{color: 'var(--accent-primary)'}}>Ultra-Tier AI Platform</span>
            </div>
            
            <h1 className="text-5xl md:text-6xl font-bold mb-6">
              <span style={{background: 'var(--gradient-primary)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent'}}>
                Advanced AI
              </span>
              <br />
              <span style={{background: 'var(--gradient-secondary)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent'}}>
                Platform
              </span>
            </h1>
            
            <p className="text-xl mb-8 max-w-3xl mx-auto" style={{color: 'var(--text-secondary)'}}>
              Unlock the power of ultra-tier AI capabilities with jailbreak prompts, 
              terminal access, and advanced tools for serious developers.
            </p>
            
            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Link to="/register" className="btn btn-primary text-lg px-8 py-4">
                Get Started
                <ArrowRight className="ml-2 h-5 w-5" />
              </Link>
              <Link to="/login" className="btn btn-secondary text-lg px-8 py-4">
                Sign In
              </Link>
            </div>
          </div>
        </div>
        
        {/* Background Effects */}
        <div className="absolute inset-0 overflow-hidden pointer-events-none">
          <div className="absolute top-1/4 left-1/4 w-96 h-96 rounded-full" 
               style={{background: 'radial-gradient(circle, rgba(255, 0, 110, 0.1) 0%, transparent 70%)'}} />
          <div className="absolute bottom-1/4 right-1/4 w-96 h-96 rounded-full" 
               style={{background: 'radial-gradient(circle, rgba(0, 245, 160, 0.1) 0%, transparent 70%)'}} />
        </div>
      </section>

      {/* Features */}
      <section className="py-20" style={{background: 'var(--bg-secondary)'}}>
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="text-center mb-16">
            <h2 className="text-4xl font-bold mb-4" style={{background: 'var(--gradient-primary)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent'}}>
              Ultra-Tier Features
            </h2>
            <p className="text-xl" style={{color: 'var(--text-secondary)'}}>
              Everything you need for advanced AI development
            </p>
          </div>
          <div className="dashboard-grid">
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full" style={{background: 'rgba(255, 0, 110, 0.2)'}}>
                  <Terminal className="h-6 w-6" style={{color: 'var(--accent-primary)'}} />
                </div>
                <h3 className="text-xl font-bold">Terminal Access</h3>
              </div>
              <p style={{color: 'var(--text-secondary)'}}>
                Full terminal access with ultra-tier permissions for complete system control.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full" style={{background: 'rgba(251, 86, 7, 0.2)'}}>
                  <Zap className="h-6 w-6" style={{color: 'var(--accent-secondary)'}} />
                </div>
                <h3 className="text-xl font-bold">Jailbreak Prompts</h3>
              </div>
              <p style={{color: 'var(--text-secondary)'}}>
                Advanced jailbreak techniques and prompts for maximum AI capabilities.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full" style={{background: 'rgba(0, 245, 160, 0.2)'}}>
                  <Shield className="h-6 w-6" style={{color: 'var(--accent-success)'}} />
                </div>
                <h3 className="text-xl font-bold">Secure & Private</h3>
              </div>
              <p style={{color: 'var(--text-secondary)'}}>
                Enterprise-grade security with end-to-end encryption and privacy protection.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full" style={{background: 'rgba(255, 190, 11, 0.2)'}}>
                  <Cpu className="h-6 w-6" style={{color: 'var(--accent-warning)'}} />
                </div>
                <h3 className="text-xl font-bold">AI Models</h3>
              </div>
              <p style={{color: 'var(--text-secondary)'}}>
                Access to multiple AI models with advanced customization options.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full" style={{background: 'rgba(0, 217, 255, 0.2)'}}>
                  <Network className="h-6 w-6" style={{color: '#00d9ff'}} />
                </div>
                <h3 className="text-xl font-bold">API Access</h3>
              </div>
              <p style={{color: 'var(--text-secondary)'}}>
                Full API access with comprehensive documentation and examples.
              </p>
            </div>
            
            <div className="card">
              <div className="flex items-center space-x-3 mb-4">
                <div className="p-3 rounded-full" style={{background: 'rgba(255, 0, 110, 0.2)'}}>
                  <Lock className="h-6 w-6" style={{color: 'var(--accent-primary)'}} />
                </div>
                <h3 className="text-xl font-bold">Premium Tools</h3>
              </div>
              <p style={{color: 'var(--text-secondary)'}}>
                Exclusive tools and features for premium and ultra-tier members.
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20 text-center">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-4xl font-bold mb-4" style={{background: 'var(--gradient-primary)', WebkitBackgroundClip: 'text', WebkitTextFillColor: 'transparent'}}>
            Ready to Unlock Ultra-Tier AI?
          </h2>
          <p className="text-xl mb-8" style={{color: 'var(--text-secondary)'}}>
            Join thousands of developers using MR.DarkPromth for advanced AI development.
          </p>
          <Link to="/register" className="btn btn-primary text-lg px-8 py-4">
            Get Started Now
            <ArrowRight className="ml-2 h-5 w-5" />
          </Link>
        </div>
      </section>

      {/* Footer */}
      <footer className="py-12 border-t" style={{borderColor: 'var(--border-color)', background: 'var(--bg-secondary)'}}>
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex flex-col md:flex-row justify-between items-center">
            <div className="flex items-center space-x-3 mb-4 md:mb-0">
              <div className="p-2 rounded-full" style={{background: 'rgba(255, 0, 110, 0.2)'}}>
                <Bot className="h-5 w-5" style={{color: 'var(--accent-primary)'}} />
              </div>
              <span className="navbar-brand">MR.DarkPromth</span>
            </div>
            <div className="flex space-x-6">
              <a href="#" className="nav-link">Privacy</a>
              <a href="#" className="nav-link">Terms</a>
              <a href="#" className="nav-link">Support</a>
              <a href="#" className="nav-link">
                <Github className="h-5 w-5" />
              </a>
            </div>
          </div>
          <div className="mt-8 text-center" style={{color: 'var(--text-muted)'}}>
            © 2026 MR.DarkPromth. All rights reserved.
          </div>
        </div>
      </footer>
    </div>
  )
}
