import React from 'react';
import './PaperCard.css';

function PaperCard({ paper }) {
  const {
    title,
    abstract,
    arxiv_id,
    arxiv_url,
    published_date,
    authors,
    implementations = []
  } = paper;

  // Parse authors from JSONB if needed
  const authorList = typeof authors === 'string' ? JSON.parse(authors) : authors;
  const authorNames = authorList ? authorList.slice(0, 3).join(', ') : '';
  const moreAuthors = authorList && authorList.length > 3 ? ` +${authorList.length - 3} more` : '';

  // Get primary implementation (official or first one)
  const primaryImpl = implementations.find(impl => impl.is_official) || implementations[0];

  // Format date
  const formattedDate = published_date
    ? new Date(published_date).toLocaleDateString('en-US', { day: 'numeric', month: 'short', year: 'numeric' })
    : '';

  // Truncate abstract
  const shortAbstract = abstract
    ? abstract.length > 200
      ? abstract.substring(0, 200) + '...'
      : abstract
    : 'No abstract available';

  return (
    <div className="paper-card">
      <div className="paper-card-image">
        {arxiv_id && (
          <a href={arxiv_url || `https://arxiv.org/abs/${arxiv_id}`} target="_blank" rel="noopener noreferrer">
            <div className="paper-thumbnail">
              <div className="paper-thumbnail-placeholder">
                <svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                  <polyline points="14 2 14 8 20 8"></polyline>
                  <line x1="16" y1="13" x2="8" y2="13"></line>
                  <line x1="16" y1="17" x2="8" y2="17"></line>
                  <polyline points="10 9 9 9 8 9"></polyline>
                </svg>
              </div>
            </div>
          </a>
        )}
      </div>

      <div className="paper-card-content">
        <h2 className="paper-title">
          {arxiv_url ? (
            <a href={arxiv_url} target="_blank" rel="noopener noreferrer">{title}</a>
          ) : (
            title
          )}
        </h2>

        <div className="paper-metadata">
          {primaryImpl && (
            <>
              <span className="paper-github">
                <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                  <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
                </svg>
                <a href={primaryImpl.github_url} target="_blank" rel="noopener noreferrer" onClick={(e) => e.stopPropagation()}>
                  {primaryImpl.github_url.replace('https://github.com/', '')}
                </a>
              </span>
              {primaryImpl.framework && (
                <span className="paper-framework-badge">
                  {primaryImpl.framework}
                </span>
              )}
            </>
          )}
          {formattedDate && (
            <span className="paper-date">• {formattedDate}</span>
          )}
        </div>

        {authorNames && (
          <div className="paper-authors">
            {authorNames}{moreAuthors}
          </div>
        )}

        <p className="paper-abstract">{shortAbstract}</p>

        {arxiv_id && (
          <div className="paper-links">
            <a href={arxiv_url || `https://arxiv.org/abs/${arxiv_id}`} target="_blank" rel="noopener noreferrer" className="paper-link-arxiv">
              arXiv:{arxiv_id}
            </a>
          </div>
        )}
      </div>
    </div>
  );
}

export default PaperCard;
